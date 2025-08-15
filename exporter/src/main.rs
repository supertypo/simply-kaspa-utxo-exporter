use chrono::{DateTime, TimeDelta, TimeZone, Timelike, Utc};
use clap::Parser;
use humantime::format_duration;
use kaspa_consensus::consensus::storage::ConsensusStorage;
use kaspa_consensus_core::config::ConfigBuilder;
use kaspa_consensus_core::constants::SOMPI_PER_KASPA;
use kaspa_consensus_core::tx::ScriptPublicKey;
use kaspa_database::prelude::{StoreError, DB};
use kaspa_txscript::extract_script_pub_key_address;
use kaspa_wrpc_client::prelude::NetworkId;
use log::{debug, error, info, trace, warn};
use regex::Regex;
use rocksdb::{DBWithThreadMode, MultiThreaded};
use simply_kaspa_utxo_exporter::signal::signal_handler::notify_on_signals;
use simply_kaspa_utxo_exporter_cli::cli_args::CliArgs;
use simply_kaspa_utxo_exporter_database::client::KaspaDbClient;
use simply_kaspa_utxo_exporter_database::models::distribution_tier::DistributionTier;
use simply_kaspa_utxo_exporter_database::models::top_script::TopScript;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::error::Error;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{env, fs};
use tokio::task;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    println!();
    println!("************************************************************");
    println!("**************** Simply Kaspa UTXO Exporter ****************");
    println!("------------------------------------------------------------");
    println!("- https://github.com/supertypo/simply-kaspa-utxo-exporter/ -");
    println!("------------------------------------------------------------");
    let cli_args = CliArgs::parse();

    env::set_var("RUST_LOG", &cli_args.log_level);
    env::set_var("RUST_LOG_STYLE", if cli_args.log_no_color { "never" } else { "always" });
    env_logger::builder().target(env_logger::Target::Stdout).format_target(false).format_timestamp_millis().init();

    let run = Arc::new(AtomicBool::new(true));
    task::spawn(notify_on_signals(run.clone()));

    info!("{} {}", env!("CARGO_PKG_NAME"), cli_args.version());
    trace!("{:?}", cli_args);

    let network_id = NetworkId::from_str(&cli_args.network).unwrap();

    let mut dbs = vec![];
    let mut last_run_ms = 0;
    for url in cli_args.database_url.clone() {
        match KaspaDbClient::new(&url).await {
            Ok(db) => {
                if let Err(e) = db.create_tables().await {
                    panic!("Failed to create tables for {url}: {e}")
                };
                if cli_args.initialize_db {
                    if let Err(e) = db.empty_tables().await {
                        panic!("Failed to empty tables for {url}: {e}")
                    };
                }
                match db.select_last_distribution_tier().await {
                    Ok(v) => {
                        if let Some(ms) = v {
                            info!("Fetched last successful run: {}", Utc.timestamp_millis_opt(ms).unwrap());
                            last_run_ms = ms;
                        }
                    }
                    Err(e) => panic!("Failed to read last run timestamp from {url}: {e}"),
                }
                dbs.push(db);
            }
            Err(e) => panic!("Database connection to {url} FAILED: {e}"),
        }
    }
    info!("Run interval is set to {} minutes", cli_args.interval_minutes);

    let run_interval = TimeDelta::minutes(cli_args.interval_minutes as i64);
    while run.load(Ordering::Relaxed) {
        let start_time = Utc::now().with_nanosecond(0).unwrap();
        let start_time_ms = start_time.timestamp_millis();
        let last_run = DateTime::from_timestamp_millis(last_run_ms).unwrap();
        let last_run_delta = start_time.signed_duration_since(last_run);
        if last_run_delta >= run_interval {
            info!("Reading tiers and top scripts");
            if last_run_ms > 0 {
                info!("Time since last run: {}", format_duration(last_run_delta.to_std().unwrap()));
            }
            let db_path = match get_db_path(cli_args.base_dir.clone(), cli_args.consensus_dir.clone(), network_id) {
                Ok(db_path) => db_path,
                Err(e) => {
                    warn!("Unable to locate consensus directory in {}: {}", cli_args.base_dir, e);
                    continue;
                }
            };
            match read_tiers_and_top_scripts(cli_args.clone(), run.clone(), network_id, db_path.clone(), start_time_ms) {
                Ok((tiers, top_scripts)) => {
                    commit_to_db_with_retry(
                        run.clone(),
                        cli_args.db_retry_count,
                        cli_args.db_retry_interval,
                        dbs.clone(),
                        &tiers,
                        &top_scripts,
                    )
                    .await;
                    last_run_ms = start_time_ms;
                    info!("Finished reading tiers and top scripts, waiting until next interval ({}m)", cli_args.interval_minutes);
                }
                Err(e) => {
                    if !run.load(Ordering::Relaxed) {
                        break;
                    }
                    error!("Failed to read tiers and top scripts, retrying in {} seconds: {e}", cli_args.data_dir_retry_interval);
                    if cli_args.data_dir_retry_interval > 3 {
                        sleep(Duration::from_secs(cli_args.data_dir_retry_interval - 3)).await;
                    }
                }
            };
        }
        sleep(Duration::from_secs(3)).await;
    }
}

fn get_db_path(base_dir: String, consensus_dir: Option<String>, network_id: NetworkId) -> std::io::Result<PathBuf> {
    let consensus_base_path = PathBuf::from(base_dir).join(network_id.to_prefixed()).join("datadir").join("consensus");

    if let Some(consensus_dir) = consensus_dir {
        let consensus_path = consensus_base_path.join(consensus_dir);
        info!("Using specified consensus directory: {}", consensus_path.display());
        Ok(consensus_path)
    } else {
        let re = Regex::new(r"^consensus-(\d+)$").unwrap();
        let consensus_dir = fs::read_dir(&consensus_base_path)?
            .filter_map(|e| e.ok())
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|name| re.is_match(name))
            .max_by(|a, b| b.cmp(a))
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No matching consensus directory found"))?;
        let consensus_path = consensus_base_path.join(consensus_dir);
        info!("Using auto-detected consensus directory: {}", consensus_path.display());
        Ok(consensus_path)
    }
}

async fn commit_to_db_with_retry(
    run: Arc<AtomicBool>,
    db_retry_count: u16,
    db_retry_interval: u64,
    dbs: Vec<KaspaDbClient>,
    tiers: &[DistributionTier],
    top_scripts: &[TopScript],
) {
    for db in dbs {
        debug!("Committing {} tiers and {} top scripts to {}", tiers.len(), top_scripts.len(), db.url_cleaned);
        for retry in 0..=db_retry_count {
            match commit_to_db(&db, tiers, top_scripts).await {
                Ok(()) => {
                    info!("Committed {} tiers and {} top scripts to {}", tiers.len(), top_scripts.len(), db.url_cleaned);
                    break;
                }
                Err(e) => {
                    let start_sleep_time = Instant::now();
                    error!("Failed to commit results to {}, retry {retry}/{db_retry_count}: {e}", db.url_cleaned);
                    while start_sleep_time.elapsed() < Duration::from_secs(db_retry_interval) {
                        if !run.load(Ordering::Relaxed) {
                            return;
                        }
                        sleep(Duration::from_secs(3)).await;
                    }
                }
            }
        }
    }
}

async fn commit_to_db(db: &KaspaDbClient, tiers: &[DistributionTier], top_scripts: &[TopScript]) -> Result<(), Box<dyn Error>> {
    db.insert_distribution_tiers(tiers).await?;
    db.insert_top_scripts(top_scripts).await?;
    Ok(())
}

fn read_tiers_and_top_scripts(
    cli_args: CliArgs,
    run: Arc<AtomicBool>,
    network_id: NetworkId,
    db_path: PathBuf,
    start_time_ms: i64,
) -> Result<(Vec<DistributionTier>, Vec<TopScript>), Box<dyn Error>> {
    let mut tiers = [(0u64, 0u64); 11]; // Covers up to 10b KAS
    let mut top_scripts_heap: BinaryHeap<Reverse<(u64, Vec<u8>)>> = BinaryHeap::with_capacity(cli_args.top_scripts_count as usize);

    for (script, amount) in read_script_amounts(run.clone(), network_id, cli_args.ignore_dust_amounts, db_path)? {
        let amount_kas = amount / SOMPI_PER_KASPA;
        let tier = ((amount_kas * 10) as f64).log10().floor() as usize;
        tiers[tier].0 += 1;
        tiers[tier].1 += amount;

        if amount_kas > cli_args.top_scripts_min_amount {
            if top_scripts_heap.len() < cli_args.top_scripts_count as usize {
                top_scripts_heap.push(Reverse((amount, script.script().to_vec())));
            } else if amount > top_scripts_heap.peek().unwrap().0 .0 {
                top_scripts_heap.pop();
                top_scripts_heap.push(Reverse((amount, script.script().to_vec())));
            }
        }
        if !run.load(Ordering::Relaxed) {
            return Err(StoreError::DataInconsistency("Shutting down".to_string()).into());
        }
    }

    let mut distribution_tiers = vec![];
    for (idx, (count, amount)) in tiers.into_iter().enumerate() {
        let amount_kas = amount / SOMPI_PER_KASPA;
        info!("Tier {idx}, count: {count}, total: {amount_kas} KAS");
        distribution_tiers.push(DistributionTier {
            tier: idx as i16,
            timestamp: start_time_ms,
            count: count as i64,
            amount: amount_kas as i64,
        });
    }

    let top_scripts = top_scripts_heap
        .into_sorted_vec()
        .into_iter()
        .enumerate()
        .map(|(idx, Reverse((amount, spk)))| {
            let amount_kas = amount / SOMPI_PER_KASPA;
            if idx < 10 {
                let prefix = kaspa_addresses::Prefix::from(network_id);
                let address = extract_script_pub_key_address(&ScriptPublicKey::from_vec(0, spk.clone()), prefix).unwrap();
                info!("Top {} address: {address}, total: {amount_kas} KAS", idx + 1);
            }
            TopScript { rank: idx as i16, timestamp: start_time_ms, script_public_key: spk, amount: amount_kas as i64 }
        })
        .collect();

    Ok((distribution_tiers, top_scripts))
}

fn read_script_amounts(
    run: Arc<AtomicBool>,
    network_id: NetworkId,
    ignore_dust_amounts: u64,
    db_path: PathBuf,
) -> Result<HashMap<ScriptPublicKey, u64>, Box<dyn Error>> {
    let mut opts = rocksdb::Options::default();
    opts.create_if_missing(false);
    opts.set_max_open_files(128);
    opts.set_allow_mmap_reads(true);
    let guard = kaspa_utils::fd_budget::acquire_guard(128).unwrap();

    info!("Reading UTXOs from VirtualStore");
    let db = Arc::new(DB::new(<DBWithThreadMode<MultiThreaded>>::open_for_read_only(&opts, db_path.to_str().unwrap(), false)?, guard));
    let config = Arc::new(ConfigBuilder::new(network_id.into()).adjust_perf_params_to_consensus_params().build());
    let storage = ConsensusStorage::new(db, config);

    let mut count = 0u64;
    let mut total_amount = 0u64;
    let mut dust_count = 0u64;
    let mut dust_total_amount = 0u64;
    let mut script_amount = HashMap::new();

    let start_time = Instant::now();
    for result in storage.virtual_stores.read().utxo_set.iterator() {
        let (_, entry) = result?;
        let amount = entry.amount;
        count += 1;
        total_amount += amount;
        if amount < ignore_dust_amounts {
            trace!("Ignoring dust UTXO of {amount} sompi");
            dust_count += 1;
            dust_total_amount += amount;
        } else {
            script_amount.entry(entry.script_public_key.clone()).and_modify(|e| *e += amount).or_insert(amount);
        }
        if count % 1_000_000 == 0 {
            info!(
                "Processed {count} UTXOs, total amount {} KAS (dust: {dust_count}/{})",
                total_amount / SOMPI_PER_KASPA,
                dust_total_amount / SOMPI_PER_KASPA,
            );
        }
        if !run.load(Ordering::Relaxed) {
            return Err(StoreError::DataInconsistency("Shutting down".to_string()).into());
        }
    }
    info!(
        "Done processing {count} UTXOs, total amount {} (dust: {dust_count}/{}), time used: {}",
        total_amount / SOMPI_PER_KASPA,
        dust_total_amount / SOMPI_PER_KASPA,
        format_duration(Duration::from_secs(start_time.elapsed().as_secs()))
    );
    Ok(script_amount)
}
