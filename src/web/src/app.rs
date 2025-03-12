use std::sync::Arc;

use axum::{extract::{Query, State}, response::IntoResponse, routing::get, Json, Router};
use common::{error::CoreError, storage::{model::{EthEventModel, EthEventType}, service::StorageService}};
use log::*;
use serde::Deserialize;


pub fn create_app<P: 'static + LogProvider + Send + Sync>(state: Arc<P>) -> Router {
    Router::new()
      .route("/logs", get(get_logs)).with_state(state)
 }
  


#[derive(Deserialize)]
struct LogQuery {
    event_type: Option<EthEventType>,
    from_id: Option<u64>,
    max: Option<u32>,
}

// This will parse query strings like `?from_id=2&max=30` into `LogQuery` structs.
async fn get_logs<P: 'static + LogProvider + Send + Sync>(State(state): State<Arc<P>>, pagination: Query<LogQuery>) -> impl IntoResponse {
    let query: LogQuery = pagination.0;
    let from_id = query.from_id.unwrap_or(0);
    let max = query.max.unwrap_or(10);
    let event_type = query.event_type.unwrap_or(EthEventType::Transfer);
    state.fetch_all_events_by_type(event_type, from_id, max).await
        .map_err(|err| {
            error!("Failed to fetch logs: {err:?}");
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })
        .map(|logs| Json(logs))
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


#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use alloy::primitives::{Address, U256};
    use axum::body::Body;
    use axum::http::{header, Method, Request, StatusCode};

    use common::storage::model::EthEventData;
    use http_body_util::BodyExt; // for `collect`
    use tower::ServiceExt; // for `call`, `oneshot`, and `ready`

    use super::*;

    #[derive(Default, Clone)]
    struct TestLogProvider {}

    impl LogProvider for TestLogProvider {
        async fn fetch_all_events_by_type(
            &self,
            event_type: EthEventType,
            from_id: u64,
            limit: u32,
        ) -> Result<Vec<EthEventModel>, CoreError> {
            // Generate 'limit` number of logs starting from `from_id`
            let logs = (from_id..from_id + (limit as u64))
                .map(|id| EthEventModel {
                    id,
                    version: 0,
                    create_epoch_millis: 0,
                    update_epoch_millis: 0,
                    data: EthEventData {
                        from: Address::random(),
                        to: Address::random(),
                        value: U256::from(id),
                        event_type: event_type.clone(),
                    },                    
                })
                .collect();
            Ok(logs)

        }
    }

    #[tokio::test]
    async fn rpc_handle_should_return_single_request() {
        // Arrange
        let app = create_app(Arc::new(TestLogProvider{}));

        // Act
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .header(header::CONTENT_TYPE, "application/json")
                    .uri("/logs")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

            assert_eq!(response.status(), StatusCode::OK);

            let body = response.into_body().collect().await.unwrap().to_bytes();
            let body: Vec<EthEventModel> = serde_json::from_slice(&body).unwrap();
            
            assert_eq!(body.len(), 10);
            assert_eq!(body[0].id, 0);
    }

}