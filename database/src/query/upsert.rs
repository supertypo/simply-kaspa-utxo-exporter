use log::trace;
use sqlx::{Error, Pool, Postgres};

use crate::models::script_utxo_count::ScriptUtxoCount;
use crate::query::insert::generate_placeholders;

pub async fn replace_script_utxo_counts(script_utxo_counts: &[ScriptUtxoCount], pool: &Pool<Postgres>) -> Result<u64, Error> {
    const COLS: usize = 3;
    const BATCH_SIZE: usize = 2_000;

    let mut total_rows = 0u64;
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM script_utxo_counts").execute(&mut *tx).await?;

    for chunk in script_utxo_counts.chunks(BATCH_SIZE) {
        let sql = format!(
            "INSERT INTO script_utxo_counts (script_public_key, script_public_key_address, count) VALUES {}",
            generate_placeholders(chunk.len(), COLS)
        );
        let mut query = sqlx::query(&sql);
        for sc in chunk {
            query = query.bind(&sc.script_public_key);
            query = query.bind(&sc.script_public_key_address);
            query = query.bind(sc.count);
        }
        total_rows += query.execute(&mut *tx).await?.rows_affected();
    }
    tx.commit().await?;
    Ok(total_rows)
}

pub async fn upsert_var(key: &str, value: &String, pool: &Pool<Postgres>) -> Result<u64, Error> {
    trace!("Saving database var with key '{}' value: {}", key, value);
    let rows_affected =
        sqlx::query("INSERT INTO vars (key, value) VALUES ($1, $2) ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value")
            .bind(key)
            .bind(value)
            .execute(pool)
            .await?
            .rows_affected();
    Ok(rows_affected)
}
