use sqlx::{Error, Pool, Postgres};

pub async fn empty_tables(pool: &Pool<Postgres>) -> Result<(), Error> {
    empty_table(pool, "distribution_tiers").await?;
    empty_table(pool, "top_scripts").await?;
    Ok(())
}

pub async fn create_tables(pool: &Pool<Postgres>) -> Result<(), Error> {
    create_distribution_tiers(pool).await?;
    create_top_scripts(pool).await?;
    Ok(())
}

async fn empty_table(pool: &Pool<Postgres>, name: &str) -> Result<(), Error> {
    if table_exists(pool, name).await? {
        sqlx::query(format!("DELETE FROM {name}").as_str()).execute(pool).await?;
    }
    Ok(())
}

async fn create_distribution_tiers(pool: &Pool<Postgres>) -> Result<(), Error> {
    if !table_exists(pool, "distribution_tiers").await? {
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
    if !table_exists(pool, "top_scripts").await? {
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

async fn table_exists(pool: &Pool<Postgres>, name: &str) -> Result<bool, Error> {
    let exists: bool = sqlx::query_scalar(
        format!(
            "SELECT EXISTS (
                SELECT 1
                FROM information_schema.tables
                WHERE table_schema = 'public'
                AND table_name = '{name}'
            )",
        )
        .as_str(),
    )
    .fetch_one(pool)
    .await?;
    Ok(exists)
}
