use c3p0::{sqlx::*, *};
use log::*;
use tokio::{sync::mpsc::UnboundedReceiver, task::JoinHandle};

use crate::{error::CoreError, subscriber::model::Event};
use ::sqlx::migrate::Migrator;

use super::{
    model::{EthEventData, EthEventModel, EthEventType, EthEventTypeDiscriminants},
    repository::EthEventRepository,
};

/// Migrator for the database. It allows to run migrations to automatically update the database.
static MIGRATOR: Migrator = ::sqlx::migrate!("resources/db/pg/migrations");

/// Service for persisting Ethereum events
pub struct StorageService {
    pool: SqlxPgC3p0Pool,
    repo: EthEventRepository,
}

impl StorageService {
    /// Creates a new instance of `StorageService`.
    ///
    /// This function initializes the service with a given Postgres connection pool
    /// and runs any pending database migrations.
    pub async fn new(pool: SqlxPgC3p0Pool) -> Result<Self, CoreError> {
        info!("StorageService - Running database migrations");
        MIGRATOR.run(pool.pool()).await?;
        info!("StorageService - Database migrations completed");
        info!("StorageService - New instance created");
        Ok(Self { pool, repo: EthEventRepository::new() })
    }

    /// Fetches all Ethereum events from the storage, optionally filtered by event type.
    /// The events are sorted in ascending order by `id`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if there is an error interacting with the database.
    pub async fn fetch_all_events(
        &self,
        event_type: Option<EthEventTypeDiscriminants>,
        from_id: u64,
        limit: u32,
    ) -> Result<Vec<EthEventModel>, CoreError> {
        debug!("StorageService - Fetching all events from the storage");
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

    /// Saves an Ethereum event to the storage.
    ///
    /// If successful, it returns the saved event model populated with the generated id.
    ///
    /// # Errors
    ///
    /// Returns `Err` if there is an error interacting with the database.
    pub async fn save_event(&self, model: EthEventData) -> Result<EthEventModel, CoreError> {
        debug!("StorageService - Saving event to the storage");
        self.pool.transaction(async |tx| self.repo.save(tx, NewModel::new(model)).await).await
    }

    /// Subscribes to an unbounded receiver of Ethereum events and saves them to the storage.
    /// The function spawns a new tokio task that listens to the input stream for the events to be persisted.
    /// It returns the join handle of the spawned task and a receiver that can be used to receive the persisted events.
    pub fn subscribe_to_event_stream(
        &self,
        mut receiver: UnboundedReceiver<Event>,
    ) -> (UnboundedReceiver<EthEventModel>, JoinHandle<()>) {
        info!("StorageService - Subscribing to event stream");

        let pool = self.pool.clone();
        let repo = self.repo.clone();
        let (response_tx, response_rx) = tokio::sync::mpsc::unbounded_channel();

        let handle = tokio::spawn(async move {
            while let Some(event) = receiver.recv().await {
                let model = match event {
                    Event::Approval { from, to, value } => {
                        EthEventData { value, event_type: EthEventType::Approve { from, to } }
                    }
                    Event::Transfer { from, to, value } => {
                        EthEventData { value, event_type: EthEventType::Transfer { from, to } }
                    }
                    Event::Deposit { to, value } => EthEventData { value, event_type: EthEventType::Deposit { to } },
                    Event::Withdrawal { from, value } => {
                        EthEventData { value, event_type: EthEventType::Withdrawal { from } }
                    }
                };
                match pool.transaction(async |tx| repo.save(tx, NewModel::new(model)).await).await {
                    Ok(event) => {
                        trace!("Event persisted in the storage: {event:?}");
                        if !response_tx.is_closed() {
                            match response_tx.send(event) {
                                Ok(()) => trace!("Response message sent"),
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
