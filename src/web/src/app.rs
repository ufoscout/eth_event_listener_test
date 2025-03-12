use std::sync::Arc;

use axum::{routing::get, Router};
use common::{error::CoreError, storage::{model::{EthEventModel, EthEventType}, service::StorageService}};


pub fn create_app<P: 'static + LogProvider + Send + Sync>(state: Arc<P>) -> Router {
    Router::new()
      .route("/", get(|| async {"hello world!"})).with_state(state)
 }
  

pub trait LogProvider {
    fn fetch_all_events_by_type(
        &self,
        event_type: EthEventType,
        from_id: u64,
        limit: u32,
    ) -> impl std::future::Future<Output = Result<Vec<EthEventModel>, CoreError>> + Send;
}

impl LogProvider for StorageService {

    async fn fetch_all_events_by_type(
        &self,
        event_type: EthEventType,
        from_id: u64,
        limit: u32,
    ) -> Result<Vec<EthEventModel>, CoreError> {
        self.fetch_all_events_by_type(event_type, from_id, limit).await
    }
}