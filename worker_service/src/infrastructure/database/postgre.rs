use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

pub async fn get_db_conn_pool(url:String) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5) // Adjust as needed
        .acquire_timeout(Duration::from_secs(3)) // Set 3-second timeout
        .connect(&url)
        .await
}