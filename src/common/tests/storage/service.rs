use common::{error::CoreError, storage::{repository::{EthEventData, EthEventRepository, EthEventType}, service::StorageService}};
use c3p0::*;
use crate::storage::new_pg_pool;


/// Tests that events can be saved and retrieved from the repository
#[tokio::test]
async fn test_eth_event_repository() {

    // Arrange
    let pool = new_pg_pool().await;
    let storage = StorageService::new(pool).await.unwrap();
    
    let mut approve_events = vec![];
    let mut transfer_events = vec![];

    // Act
    {
        // insert 10 Approve events
        for i in 0..10 {
            approve_events.push(storage.save_event(EthEventData {
                event_type: EthEventType::Approve,
                from: i.to_string(),
                to: i.to_string()
            }).await.unwrap());
        }
        
                // insert 10 Trasfer events
        for i in 0..10 {
            transfer_events.push(storage.save_event(EthEventData {
                event_type: EthEventType::Transfer,
                from: i.to_string(),
                to: i.to_string()
            }).await.unwrap());
        }

    }

    // Assert
    let approve_first_id = approve_events[0].id;
    let approve_events_from_storage = storage.fetch_all_events_by_type(EthEventType::Approve, approve_first_id, 10).await.unwrap();
    assert_eq!(approve_events, approve_events_from_storage);

    let transfer_first_id = transfer_events[0].id;
    let transfer_events_from_storage = storage.fetch_all_events_by_type(EthEventType::Transfer, transfer_first_id, 10).await.unwrap();
    assert_eq!(transfer_events, transfer_events_from_storage);


}