use c3p0::sqlx::*;
use c3p0::*;
use serde::{Deserialize, Serialize};
use ::sqlx::PgConnection;
use strum::{AsRefStr, Display};

use crate::error::CoreError;

pub type EthEventModel = Model<u64, EthEventData>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EthEventData {
    pub from: String,
    pub to: String,
    pub event_type: EthEventType
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, AsRefStr, Display)]
pub enum EthEventType {
    Approve,
    Transfer,
}


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
    pub fn new() -> Self {
        Self { repo: SqlxPgC3p0JsonBuilder::new("ETH_EVENT").build() }
    }

    pub async fn fetch_all_by_type(
        &self,
        tx: &mut PgConnection,
        event_type: EthEventType,
        from_id: &u64,
        limit: u32,
    ) -> Result<Vec<EthEventModel>, CoreError> {
        let sql = format!(
            r#"
            {}
            where id >= $1 and DATA ->> 'event_type' = $2
            order by id asc
            limit $3
        "#,
            self.repo.queries().find_base_sql_query
        );

        Ok(self
            .repo
            .fetch_all_with_sql(
                tx,
                self.repo.query_with_id(&sql, from_id).bind(event_type.as_ref()).bind(limit as i64),
            )
            .await?)
    }

    pub async fn save(&self, tx: &mut PgConnection, model: NewModel<EthEventData>) -> Result<EthEventModel, CoreError> {
        Ok(self.repo.save(tx, model).await?)
    }
}

