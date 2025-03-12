use common::{error::CoreError, storage::{repository::{EthEventData, EthEventRepository, EthEventType}, service::StorageService}};
use c3p0::*;
use crate::storage::new_pg_pool;


/// Tests that events can be saved and retrieved from the repository
#[tokio::test]
async fn test_eth_event_repository() {

    // Arrange
    let pool = new_pg_pool().await;
    let storage = StorageService::new(pool).await.unwrap();
    
    // Act
       
        // insert 10 Trasfer events
        for i in 0..10 {
            storage.save_event(EthEventData {
                event_type: EthEventType::Transfer,
                from: i.to_string(),
                to: i.to_string()
            }).await.unwrap();
        }

        // insert 10 Approve events
        for i in 0..10 {
            storage.save_event(EthEventData {
                event_type: EthEventType::Approve,
                from: i.to_string(),
                to: i.to_string()
            }).await.unwrap();
        }


}