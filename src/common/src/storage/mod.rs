use sqlx::migrate::Migrator;

pub mod repository;
pub mod service;

static MIGRATOR: Migrator = ::sqlx::migrate!("resources/db/pg/migrations");

pub async fn run_migrations(pool: &sqlx::Pool<sqlx::Postgres>) -> sqlx::Result<()> {
    Ok(MIGRATOR.run(pool).await?)
}


