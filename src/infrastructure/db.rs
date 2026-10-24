use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

use crate::config::Config;
use crate::error::AppResult;

pub async fn create_pool(config: &Config) -> AppResult<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&config.database_url)
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> AppResult<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await?;
    Ok(())
}
