use ::sqlx::PgConnection;
use c3p0::sqlx::*;
use c3p0::*;
use log::trace;

use crate::error::CoreError;

use super::model::{EthEventData, EthEventModel, EthEventTypeDiscriminants};

/// An Ethereum event repository that persists events in the ETH_EVENT table of a Postgres database
#[derive(Clone)]
pub struct EthEventRepository {
    repo: SqlxPgC3p0Json<u64, EthEventData, DefaultJsonCodec>,
}

impl Default for EthEventRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl EthEventRepository {
    /// Create a new EthEventRepository
    pub fn new() -> Self {
        Self { repo: SqlxPgC3p0JsonBuilder::new("ETH_EVENT").build() }
    }

    /// Fetches all Ethereum events from the database starting from the given `from_id` up to `limit` events.
    ///
    /// The events are sorted in ascending order by `id`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if there is an error interacting with the database.
    pub async fn fetch_all(
        &self,
        tx: &mut PgConnection,
        from_id: &u64,
        limit: u32,
    ) -> Result<Vec<EthEventModel>, CoreError> {
        trace!("Fetching all events from the database, from id: {}, limit: {}", from_id, limit);
        let sql = format!(
            r#"
            {}
            where id >= $1
            order by id asc
            limit $2
        "#,
            self.repo.queries().find_base_sql_query
        );

        Ok(self.repo.fetch_all_with_sql(tx, self.repo.query_with_id(&sql, from_id).bind(limit as i64)).await?)
    }

    /// Fetches all Ethereum events of a given type from the database starting from the given `from_id` up to `limit` events.
    ///
    /// The events are sorted in ascending order by `id`.
    ///
    /// # Errors
    ///
    /// Returns `Err` if there is an error interacting with the database.
    pub async fn fetch_all_by_type(
        &self,
        tx: &mut PgConnection,
        event_type: EthEventTypeDiscriminants,
        from_id: &u64,
        limit: u32,
    ) -> Result<Vec<EthEventModel>, CoreError> {
        trace!("Fetching all events of type {} from the database, from id: {}, limit: {}", event_type, from_id, limit);

        let sql = format!(
            r#"
            {}
            where id >= $1 and DATA -> 'event_type' ->> 'type' = $2
            order by id asc
            limit $3
        "#,
            self.repo.queries().find_base_sql_query
        );

        Ok(self
            .repo
            .fetch_all_with_sql(tx, self.repo.query_with_id(&sql, from_id).bind(event_type.as_ref()).bind(limit as i64))
            .await?)
    }

    /// Saves an Ethereum event to the database.
    /// If successful, it returns the saved event model populated with the generated id.
    ///
    /// # Errors
    ///
    /// Returns `Err` if there is an error interacting with the database.
    pub async fn save(&self, tx: &mut PgConnection, model: NewModel<EthEventData>) -> Result<EthEventModel, CoreError> {
        trace!("Saving event to the database: {:?}", model);
        Ok(self.repo.save(tx, model).await?)
    }
}
