use std::str::FromStr;
use std::time::Duration;

use log::{debug, info, LevelFilter};
use regex::Regex;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{ConnectOptions, Error, Pool, Postgres};

use crate::models::distribution_tier::DistributionTier;
use crate::models::top_script::TopScript;
use crate::query;

#[derive(Clone)]
pub struct KaspaDbClient {
    pool: Pool<Postgres>,
}

impl KaspaDbClient {
    pub async fn new(url: &str) -> Result<KaspaDbClient, Error> {
        Self::new_with_args(url, 10).await
    }

    pub async fn new_with_args(url: &str, pool_size: u32) -> Result<KaspaDbClient, Error> {
        let url_cleaned = Regex::new(r"(postgres://postgres:)[^@]+(@)").expect("Failed to parse url").replace(url, "$1$2");
        debug!("Connecting to PostgreSQL {}", url_cleaned);
        let connect_opts = PgConnectOptions::from_str(url)?.log_slow_statements(LevelFilter::Warn, Duration::from_secs(60));
        let pool = PgPoolOptions::new()
            .acquire_timeout(Duration::from_secs(10))
            .max_connections(pool_size)
            .connect_with(connect_opts)
            .await?;
        info!("Connected to PostgreSQL {}", url_cleaned);
        Ok(KaspaDbClient { pool })
    }

    pub async fn close(&mut self) -> Result<(), Error> {
        self.pool.close().await;
        Ok(())
    }

    pub async fn create_tables(&self) -> Result<(), Error> {
        query::create::create_tables(&self.pool).await
    }

    pub async fn select_var(&self, key: &str) -> Result<String, Error> {
        query::select::select_var(key, &self.pool).await
    }

    pub async fn upsert_var(&self, key: &str, value: &String) -> Result<u64, Error> {
        query::upsert::upsert_var(key, value, &self.pool).await
    }

    pub async fn select_last_distribution_tier(&self) -> Result<i64, Error> {
        query::select::select_last_distribution_tier(&self.pool).await
    }

    pub async fn insert_distribution_tiers(&self, distribution_tiers: &[DistributionTier]) -> Result<u64, Error> {
        query::insert::insert_distribution_tiers(distribution_tiers, &self.pool).await
    }

    pub async fn insert_top_scripts(&self, insert_top_scripts: &[TopScript]) -> Result<u64, Error> {
        query::insert::insert_top_scripts(insert_top_scripts, &self.pool).await
    }
}
