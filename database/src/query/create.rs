use sqlx::{Error, Pool, Postgres};

pub async fn create_tables(pool: &Pool<Postgres>) -> Result<(), Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS distribution_tiers (
            timestamp BIGINT,
            tier SMALLINT,
            count BIGINT,
            amount BIGINT,
            PRIMARY KEY (timestamp, tier)
        )",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS top_scripts (
            timestamp BIGINT,
            rank SMALLINT,
            script_public_key BYTEA,
            amount BIGINT,
            PRIMARY KEY (timestamp, rank)
        )",
    )
    .execute(pool)
    .await?;
    Ok(())
}
