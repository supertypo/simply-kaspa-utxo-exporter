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
    const COLS: usize = 4;
    let sql = format!(
        "INSERT INTO top_scripts (timestamp, rank, script_public_key, amount) VALUES {} ON CONFLICT DO NOTHING",
        generate_placeholders(top_scripts.len(), COLS)
    );
    let mut query = sqlx::query(&sql);
    for ts in top_scripts {
        query = query.bind(ts.timestamp);
        query = query.bind(ts.rank);
        query = query.bind(&ts.script_public_key);
        query = query.bind(ts.amount);
    }
    Ok(query.execute(pool).await?.rows_affected())
}

fn generate_placeholders(rows: usize, columns: usize) -> String {
    (0..rows).map(|i| format!("({})", (1..=columns).map(|c| format!("${}", c + i * columns)).join(", "))).join(", ")
}
