use sqlx::{Error, Pool, Postgres, Row};

pub async fn select_var(key: &str, pool: &Pool<Postgres>) -> Result<String, Error> {
    sqlx::query("SELECT value FROM vars WHERE key = $1").bind(key).fetch_one(pool).await?.try_get(0)
}

pub async fn select_last_distribution_tier(pool: &Pool<Postgres>) -> Result<i64, Error> {
    Ok(sqlx::query_scalar::<_, Option<i64>>("SELECT max(timestamp) FROM distribution_tiers WHERE tier = 0")
        .fetch_one(pool)
        .await?
        .unwrap_or(0))
}
