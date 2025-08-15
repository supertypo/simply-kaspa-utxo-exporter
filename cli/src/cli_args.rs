use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Clone, Debug, Serialize, Deserialize)]
#[command(name = "simply-kaspa-utxo-exporter", version = env!("VERGEN_GIT_DESCRIBE"))]
#[serde(rename_all = "camelCase")]
pub struct CliArgs {
    #[clap(short, long, default_value = "mainnet", help = "The network type and suffix, e.g. 'testnet-10'")]
    pub network: String,
    #[clap(short, long, default_value = "~/.rusty-kaspa", help = "Kaspad data base directory")]
    pub base_dir: String,
    #[clap(long, help = "Kaspad consensus subdir, leave empty to auto-detect")]
    pub consensus_dir: Option<String>,
    #[clap(short, long, default_value = "postgres://postgres:postgres@localhost:5432/postgres", help = "PostgreSQL url(s)")]
    pub database_url: Vec<String>,
    #[clap(short, long, default_value = "60", help = "Interval between utxo set rescanning (0 = oneshot")]
    pub interval_minutes: u64,
    #[clap(long, default_value = "10000", help = "Ignore utxos with amounts less than this (in sompi/litra)")]
    pub ignore_dust_amounts: u64,
    #[clap(long, default_value = "1000", help = "Number of top scripts to index (0 = unlimited)")]
    pub top_scripts_count: u64,
    #[clap(long, default_value = "100000", help = "The minimum balance to be considered for top-n list")]
    pub top_scripts_min_amount: u64,
    #[clap(long, default_value = "120", help = "Interval between datadir read retries (in seconds)")]
    pub data_dir_retry_interval: u64,
    #[clap(long, default_value = "30", help = "Interval between db retries (in seconds)")]
    pub db_retry_interval: u64,
    #[clap(long, default_value = "20", help = "How many times to retry commit to db before moving on")]
    pub db_retry_count: u16,
    #[clap(short = 'c', long, help = "Empties the tables. Use with care")]
    pub initialize_db: bool,
    #[clap(short, long, default_value = "info", help = "error, warn, info, debug, trace, off")]
    pub log_level: String,
    #[clap(long, help = "Disable colored output")]
    pub log_no_color: bool,
}

impl CliArgs {
    pub fn version(&self) -> String {
        env!("VERGEN_GIT_DESCRIBE").to_string()
    }

    pub fn commit_id(&self) -> String {
        env!("VERGEN_GIT_SHA").to_string()
    }
}
