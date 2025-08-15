use itertools::Itertools;
use sqlx::{Error, Pool, Postgres};

use crate::models::distribution_tier::DistributionTier;
use crate::models::top_script::TopScript;

pub async fn insert_distribution_tiers(distribution_tiers: &[DistributionTier], pool: &Pool<Postgres>) -> Result<u64, Error> {
    const COLS: usize = 4;
    let sql = format!(
        "INSERT INTO distribution_tiers (timestamp, tier, count, amount) VALUES {} ON CONFLICT DO NOTHING",
        generate_placeholders(distribution_tiers.len(), COLS)
    );
    let mut query = sqlx::query(&sql);
    for dt in distribution_tiers {
        query = query.bind(dt.timestamp);
        query = query.bind(dt.tier);
        query = query.bind(dt.count);
        query = query.bind(dt.amount);
    }
    Ok(query.execute(pool).await?.rows_affected())
}

pub async fn insert_top_scripts(top_scripts: &[TopScript], pool: &Pool<Postgres>) -> Result<u64, Error> {
    const COLS: usize = 5;
    const BATCH_SIZE: usize = 2_000;

    let mut total_rows = 0u64;
    let mut tx = pool.begin().await?;

    for top_scripts_chunk in top_scripts.chunks(BATCH_SIZE) {
        let sql = format!(
            "INSERT INTO top_scripts (timestamp, rank, script_public_key, script_public_key_address, amount)
             VALUES {} ON CONFLICT DO NOTHING",
            generate_placeholders(top_scripts_chunk.len(), COLS)
        );
        let mut query = sqlx::query(&sql);
        for ts in top_scripts_chunk {
            query = query.bind(ts.timestamp);
            query = query.bind(ts.rank);
            query = query.bind(&ts.script_public_key);
            query = query.bind(&ts.script_public_key_address);
            query = query.bind(ts.amount);
        }
        total_rows += query.execute(&mut *tx).await?.rows_affected();
    }
    tx.commit().await?;
    Ok(total_rows)
}

fn generate_placeholders(rows: usize, columns: usize) -> String {
    (0..rows).map(|i| format!("({})", (1..=columns).map(|c| format!("${}", c + i * columns)).join(", "))).join(", ")
}
