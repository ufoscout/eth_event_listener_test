use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
};
use common::{
    error::CoreError,
    storage::{
        model::{EthEventModel, EthEventTypeDiscriminants},
        service::StorageService,
    },
};
use log::*;
use serde::Deserialize;

pub fn create_app<P: 'static + LogProvider + Send + Sync>(state: Arc<P>) -> Router {
    Router::new().route("/logs", get(get_logs)).with_state(state)
}

#[derive(Deserialize)]
struct LogQuery {
    event_type: Option<EthEventTypeDiscriminants>,
    from_id: Option<u64>,
    max: Option<u32>,
}

// This will parse query strings like `?from_id=2&max=30` into `LogQuery` structs.
async fn get_logs<P: 'static + LogProvider + Send + Sync>(
    State(state): State<Arc<P>>,
    pagination: Query<LogQuery>,
) -> impl IntoResponse {
    let query: LogQuery = pagination.0;
    let from_id = query.from_id.unwrap_or(0);
    let max = query.max.unwrap_or(10);
    state
        .fetch_all_events(query.event_type, from_id, max)
        .await
        .map_err(|err: CoreError| {
            error!("Failed to fetch logs: {err:?}");
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        })
        .map(Json)
}

pub trait LogProvider {
    fn fetch_all_events(
        &self,
        event_type: Option<EthEventTypeDiscriminants>,
        from_id: u64,
        limit: u32,
    ) -> impl std::future::Future<Output = Result<Vec<EthEventModel>, CoreError>> + Send;
}

impl LogProvider for StorageService {
    async fn fetch_all_events(
        &self,
        event_type: Option<EthEventTypeDiscriminants>,
        from_id: u64,
        limit: u32,
    ) -> Result<Vec<EthEventModel>, CoreError> {
        self.fetch_all_events(event_type, from_id, limit).await
    }
}

#[cfg(test)]
mod tests {

    use std::sync::Arc;

    use alloy::primitives::{Address, U256};
    use axum::body::Body;
    use axum::http::{Method, Request, StatusCode, header};

    use common::storage::model::{EthEventData, EthEventType};
    use http_body_util::BodyExt; // for `collect`
    use tower::ServiceExt; // for `call`, `oneshot`, and `ready`

    use super::*;

    #[derive(Default, Clone)]
    struct TestLogProvider {}

    impl LogProvider for TestLogProvider {
        async fn fetch_all_events(
            &self,
            event_type: Option<EthEventTypeDiscriminants>,
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
                        value: U256::from(id),
                        event_type: event_type
                            .clone()
                            .map(|typ| match typ {
                                EthEventTypeDiscriminants::Approve => {
                                    EthEventType::Approve { from: Address::random(), to: Address::random() }
                                }
                                EthEventTypeDiscriminants::Transfer => {
                                    EthEventType::Transfer { from: Address::random(), to: Address::random() }
                                }
                                EthEventTypeDiscriminants::Deposit => EthEventType::Deposit { to: Address::random() },
                                EthEventTypeDiscriminants::Withdrawal => {
                                    EthEventType::Withdrawal { from: Address::random() }
                                }
                            })
                            .unwrap_or_else(|| match id % 4 {
                                0 => EthEventType::Approve { from: Address::random(), to: Address::random() },
                                1 => EthEventType::Transfer { from: Address::random(), to: Address::random() },
                                2 => EthEventType::Deposit { to: Address::random() },
                                _ => EthEventType::Withdrawal { from: Address::random() },
                            }),
                    },
                })
                .collect();
            Ok(logs)
        }
    }

    #[tokio::test]
    async fn test_app_return_logs_with_default_query_values() {
        // Arrange
        let app = create_app(Arc::new(TestLogProvider {}));

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

        // The type is not specified, then check that the event types are alternating between `Approve` and `Transfer`
        for log in body.iter() {
            assert_eq!(
                match log.id % 4 {
                    0 => EthEventTypeDiscriminants::Approve,
                    1 => EthEventTypeDiscriminants::Transfer,
                    2 => EthEventTypeDiscriminants::Deposit,
                    _ => EthEventTypeDiscriminants::Withdrawal,
                },
                log.data.event_type.clone().into(),
            );
        }
    }

    #[tokio::test]
    async fn test_app_return_logs_with_custom_query_values() {
        // Arrange
        let app = create_app(Arc::new(TestLogProvider {}));

        // Act
        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .header(header::CONTENT_TYPE, "application/json")
                    .uri("/logs?from_id=1234&max=55&event_type=Transfer")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body: Vec<EthEventModel> = serde_json::from_slice(&body).unwrap();

        assert_eq!(body.len(), 55);
        assert_eq!(body[0].id, 1234);

        // The type is specified as `Transfer`, then check that all event types are `Transfer`
        for log in body {
            assert_eq!(EthEventTypeDiscriminants::Transfer, log.data.event_type.clone().into());
        }
    }
}
