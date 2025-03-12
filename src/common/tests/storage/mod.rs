use crate::get_settings;
use c3p0::sqlx::SqlxPgC3p0Pool;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

mod service;

pub async fn new_pg_pool() -> SqlxPgC3p0Pool {
    let settings = get_settings().database;

    let options = PgConnectOptions::new()
        .username(&settings.username)
        .password(&settings.password)
        .database(&settings.database)
        .host(&settings.host)
        .port(settings.port);

    let pool = PgPoolOptions::new().max_connections(settings.max_connections).connect_with(options).await.unwrap();

    SqlxPgC3p0Pool::new(pool)
}
