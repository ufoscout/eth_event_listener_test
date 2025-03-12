use c3p0::{sqlx::*, *};

use crate::error::CoreError;
use ::sqlx::migrate::Migrator;

use super::repository::{EthEventData, EthEventModel, EthEventRepository, EthEventType};

static MIGRATOR: Migrator = ::sqlx::migrate!("resources/db/pg/migrations");

pub struct StorageService {
    pool: SqlxPgC3p0Pool,
    repo: EthEventRepository,
}

impl StorageService {
    pub async fn new(pool: SqlxPgC3p0Pool) -> Result<Self, CoreError> {
        MIGRATOR.run(pool.pool()).await?;
        Ok(Self { pool, repo: EthEventRepository::new() })
    }

    pub async fn fetch_all_events_by_type(
        &self,
        event_type: EthEventType,
        from_id: u64,
        limit: u32,
    ) -> Result<Vec<EthEventModel>, CoreError> {
        self.pool.transaction(async |tx| self.repo.fetch_all_by_type(tx, event_type, &from_id, limit).await).await
    }

    pub async fn save_event(&self, model: EthEventData) -> Result<EthEventModel, CoreError> {
        self.pool.transaction(async |tx| self.repo.save(tx, NewModel::new(model)).await).await
    }
}
