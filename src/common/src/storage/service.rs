use c3p0::{sqlx::*, *};
use log::*;
use tokio::{sync::mpsc::UnboundedReceiver, task::JoinHandle};

use crate::{error::CoreError, subscriber::model::Event};
use ::sqlx::migrate::Migrator;

use super::{
    model::{EthEventData, EthEventModel, EthEventType},
    repository::EthEventRepository,
};

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

    pub async fn fetch_all_events(
        &self,
        event_type: Option<EthEventType>,
        from_id: u64,
        limit: u32,
    ) -> Result<Vec<EthEventModel>, CoreError> {
        self.pool
            .transaction(async |tx| {
                if let Some(event_type) = event_type {
                    self.repo.fetch_all_by_type(tx, event_type, &from_id, limit).await
                } else {
                    self.repo.fetch_all(tx, &from_id, limit).await
                }
            })
            .await
    }

    pub async fn save_event(&self, model: EthEventData) -> Result<EthEventModel, CoreError> {
        self.pool.transaction(async |tx| self.repo.save(tx, NewModel::new(model)).await).await
    }

    pub fn subscribe_to_event_stream(
        &self,
        mut receiver: UnboundedReceiver<Event>,
    ) -> (UnboundedReceiver<EthEventModel>, JoinHandle<()>) {
        let pool = self.pool.clone();
        let repo = self.repo.clone();
        let (response_tx, response_rx) = tokio::sync::mpsc::unbounded_channel();

        let handle = tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                let model = match event {
                    Event::Approval { from, to, value } => {
                        EthEventData { from, to, value, event_type: EthEventType::Approve }
                    }
                    Event::Transfer { from, to, value } => {
                        EthEventData { from, to, value, event_type: EthEventType::Transfer }
                    }
                };
                match pool.transaction(async |tx| repo.save(tx, NewModel::new(model)).await).await {
                    Ok(event) => {
                        debug!("Event persisted in the storage: {event:?}");
                        if !response_tx.is_closed() {
                            match response_tx.send(event) {
                                Ok(()) => debug!("Response message sent"),
                                Err(err) => error!("Failed to send response message: {err:?}"),
                            }
                        }
                    }
                    Err(err) => error!("Failed to persist new event: {err:?}"),
                };
            }
        });

        (response_rx, handle)
    }
}
