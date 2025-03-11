use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use crate::get_settings;

pub async fn new_pg_pool() -> sqlx::Pool<sqlx::Postgres> {

    let settings = get_settings().database;

    let options = PgConnectOptions::new()
        .username(&settings.username)
        .password(&settings.password)
        .database(&settings.database)
        .host(&settings.host)
        .port(settings.port);

    let pool = PgPoolOptions::new()
        .max_connections(settings.max_connections)
        .connect_with(options)
        .await
        .unwrap();

    pool
}

/// Tests that the database can be connected to
#[tokio::test]
async fn test_connection() {
    let pool = new_pg_pool().await;
    // assert!(pool.is_ok());
}