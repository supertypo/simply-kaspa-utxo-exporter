use sqlx::{Error, Pool, Postgres};

pub async fn create_tables(pool: &Pool<Postgres>) -> Result<(), Error> {
    create_distribution_tiers(pool).await?;
    create_top_scripts(pool).await?;
    Ok(())
}

async fn create_distribution_tiers(pool: &Pool<Postgres>) -> Result<(), Error> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1
            FROM information_schema.tables
            WHERE table_schema = 'public'
            AND table_name = 'distribution_tiers'
        )",
    )
    .fetch_one(pool)
    .await?;

    if !exists {
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
    }
    Ok(())
}

async fn create_top_scripts(pool: &Pool<Postgres>) -> Result<(), Error> {
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS (
            SELECT 1
            FROM information_schema.tables
            WHERE table_schema = 'public'
            AND table_name = 'top_scripts'
        )",
    )
    .fetch_one(pool)
    .await?;

    if !exists {
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
    }
    Ok(())
}
