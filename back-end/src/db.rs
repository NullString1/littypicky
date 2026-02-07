use sqlx::{postgres::PgPoolOptions, PgPool};
use crate::config::Config;

pub async fn create_pool(config: &Config) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database.url)
        .await
}
