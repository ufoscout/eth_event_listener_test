use std::str::FromStr;

use crate::storage::new_pg_pool;
use alloy::primitives::{Address, U256};
use common::{storage::{
    model::{EthEventData, EthEventType}, service::StorageService
}, subscriber::model::Event};
use rand::{random, seq::{IndexedRandom, SliceRandom}};

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
            approve_events.push(
                storage
                    .save_event(EthEventData {
                        event_type: EthEventType::Approve,
                        from: i.to_string(),
                        to: i.to_string(),
                    })
                    .await
                    .unwrap(),
            );
        }

        // insert 10 Trasfer events
        for i in 0..10 {
            transfer_events.push(
                storage
                    .save_event(EthEventData {
                        event_type: EthEventType::Transfer,
                        from: i.to_string(),
                        to: i.to_string(),
                    })
                    .await
                    .unwrap(),
            );
        }
    }

    // Assert - fetch Approve events
    {
        let approve_first_id = approve_events[0].id;

        let approve_events_from_storage =
            storage.fetch_all_events_by_type(EthEventType::Approve, approve_first_id, 10).await.unwrap();
        assert_eq!(approve_events, approve_events_from_storage);

        // Fetch with offset and limit
        let approve_events_from_storage =
            storage.fetch_all_events_by_type(EthEventType::Approve, approve_first_id + 1, 4).await.unwrap();
        assert_eq!(approve_events[1..5], approve_events_from_storage);
    }

    // Assert - fetch Transfer events
    {
        let transfer_first_id = transfer_events[0].id;

        let transfer_events_from_storage =
            storage.fetch_all_events_by_type(EthEventType::Transfer, transfer_first_id, 10).await.unwrap();
        assert_eq!(transfer_events, transfer_events_from_storage);

        // Fetch with offset and limit
        let transfer_events_from_storage =
            storage.fetch_all_events_by_type(EthEventType::Transfer, transfer_first_id + 3, 3).await.unwrap();
        assert_eq!(transfer_events[3..6], transfer_events_from_storage);
    }
}


/// Tests that events can be saved and retrieved from the repository
#[tokio::test]
async fn test_save_events_from_receiver_stream() {
    // Arrange
    let pool = new_pg_pool().await;
    let storage = StorageService::new(pool).await.unwrap();
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    let events_count = 50;
    let mut sent_events = vec![];
    let mut received_events = vec![];

    // Act
    let (mut response_rx, _handle) = storage.subscribe_to_event_stream(rx);

        // simulate 50 random events
        for i in 0..events_count {

            let event = Event::Approval {
                from: Address::random(),
                to: Address::random(),
                value: U256::from(random::<u64>()),
            };

            sent_events.push(event.clone());
            tx.send(event).unwrap();
        }

        // wait for all events to be processed
        for _ in 0..events_count {
            received_events.push(response_rx.recv().await.unwrap());
        }

    // Assert

    assert_eq!(sent_events.len(), received_events.len());

    for (sent, received) in sent_events.iter().zip(received_events.iter()) {
        match sent {
            Event::Approval { from, to, value } => {
                assert_eq!(from, &Address::from_str(&received.data.from).unwrap());
                assert_eq!(to, &Address::from_str(&received.data.to).unwrap());
                assert_eq!(EthEventType::Approve, received.data.event_type);
            },
            Event::Transfer { from, to, value } => {
                assert_eq!(from, &Address::from_str(&received.data.from).unwrap());
                assert_eq!(to, &Address::from_str(&received.data.to).unwrap());
                assert_eq!(EthEventType::Transfer, received.data.event_type);
            },
        }
    }

    // Assert that all events are persisted
    for event in received_events.iter() {
        let fetched_event = storage.fetch_all_events_by_type(event.data.event_type.clone(), event.id, 1).await.unwrap();
        assert_eq!(event, &fetched_event[0]);
    }
}