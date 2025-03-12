use c3p0::C3p0Error;
use sqlx::migrate::MigrateError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CoreError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Database Migration error: {0}")]
    DatabaseMigrationError(String),
}

impl From<C3p0Error> for CoreError {
    fn from(err: C3p0Error) -> Self {
        CoreError::DatabaseError(format!("{:?}", err))
    }
}

impl From<MigrateError> for CoreError {
    fn from(err: MigrateError) -> Self {
        CoreError::DatabaseMigrationError(format!("{:?}", err))
    }
}
