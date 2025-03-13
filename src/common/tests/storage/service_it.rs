use crate::storage::new_pg_pool;
use alloy::primitives::{Address, U256};
use common::{
    storage::{
        model::{EthEventData, EthEventType, EthEventTypeDiscriminants},
        service::StorageService,
    },
    subscriber::model::Event,
};
use rand::random;

/// Tests that events can be saved and retrieved from the repository
#[tokio::test]
async fn test_eth_event_storage() {
    // Arrange
    let pool = new_pg_pool().await;
    let storage = StorageService::new(pool).await.unwrap();

    let mut approve_events = vec![];
    let mut transfer_events = vec![];

    // Act
    {
        // insert 10 Approve events
        for _ in 0..10 {
            approve_events.push(
                storage
                    .save_event(EthEventData {
                        event_type: EthEventType::Approve {
                            from: Address::random(),
                            to: Address::random(),
                        },
                        value: U256::from(random::<u64>()),
                    })
                    .await
                    .unwrap(),
            );
        }

        // insert 10 Trasfer events
        for _ in 0..10 {
            transfer_events.push(
                storage
                    .save_event(EthEventData {
                        event_type: EthEventType::Transfer {
                            from: Address::random(),
                            to: Address::random(),
                        },
                        value: U256::from(random::<u64>()),
                    })
                    .await
                    .unwrap(),
            );
        }
    }

    // Assert - fetch Approve logs
    {
        let approve_first_id = approve_events[0].id;

        let approve_events_from_storage =
            storage.fetch_all_events(Some(EthEventTypeDiscriminants::Approve), approve_first_id, 10).await.unwrap();
        assert_eq!(approve_events_from_storage.len(), 10);
        assert_eq!(approve_first_id, approve_events_from_storage[0].id);

        // check that all events have type Approve
        for event in approve_events_from_storage.iter() {
            assert_eq!(EthEventTypeDiscriminants::Approve, event.data.event_type.clone().into());
        }
    }

    // Assert - Fetch Approve logs with offset and limit
    {
        let approve_first_id = approve_events[1].id;

        let approve_events_from_storage =
            storage.fetch_all_events(Some(EthEventTypeDiscriminants::Approve), approve_first_id, 4).await.unwrap();

        assert_eq!(approve_events_from_storage.len(), 4);
        assert_eq!(approve_first_id, approve_events_from_storage[0].id);

        // check that all events have type Approve
        for event in approve_events_from_storage.iter() {
            assert_eq!(EthEventTypeDiscriminants::Approve, event.data.event_type.clone().into());
        }
    }

    // Assert - fetch Transfer events
    {
        let transfer_first_id = transfer_events[0].id;

        let transfer_events_from_storage =
            storage.fetch_all_events(Some(EthEventTypeDiscriminants::Transfer), transfer_first_id, 10).await.unwrap();

        assert_eq!(transfer_events_from_storage.len(), 10);
        assert_eq!(transfer_first_id, transfer_events_from_storage[0].id);

        // check that all events have type Approve
        for event in transfer_events_from_storage.iter() {
            assert_eq!(EthEventTypeDiscriminants::Transfer, event.data.event_type.clone().into());
        }
    }

    // Assert - Fetch Transfer Logs with offset and limit
    {
        let transfer_first_id = transfer_events[0].id;
        let transfer_events_from_storage =
            storage.fetch_all_events(Some(EthEventTypeDiscriminants::Transfer), transfer_first_id, 3).await.unwrap();

        assert_eq!(transfer_events_from_storage.len(), 3);
        assert_eq!(transfer_first_id, transfer_events_from_storage[0].id);

        // check that all events have type Approve
        for event in transfer_events_from_storage.iter() {
            assert_eq!(EthEventTypeDiscriminants::Transfer, event.data.event_type.clone().into());
        }
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
    for _ in 0..events_count {
        {
            let event =
                Event::Approval { from: Address::random(), to: Address::random(), value: U256::from(random::<u64>()) };
                sent_events.push(event.clone());
            tx.send(event).unwrap();
        }
        {
            let event =
                Event::Transfer { from: Address::random(), to: Address::random(), value: U256::from(random::<u64>()) };
                sent_events.push(event.clone());
            tx.send(event).unwrap();
        }
        {
            let event = Event::Deposit { to: Address::random(), value: U256::from(random::<u64>()) };
            sent_events.push(event.clone());
            tx.send(event).unwrap();
        }
        {
            let event = Event::Withdrawal { from: Address::random(), value: U256::from(random::<u64>()) };
            sent_events.push(event.clone());
            tx.send(event).unwrap();
        }
    }

    // Drop the sender to close the channel
    drop(tx);

    // wait for all events to be processed until the channel is closed
    while let Some(event) = response_rx.recv().await {
        received_events.push(event);
    }

    // Assert

    assert_eq!(sent_events.len(), received_events.len());

    for (sent, received) in sent_events.iter().zip(received_events.iter()) {
        match sent {
            Event::Approval{from,to,value}=>{
                assert_eq!(value, &received.data.value);
                assert_eq!(EthEventType::Approve { 
                    from: from.to_owned(),
                    to: to.to_owned()
                }, received.data.event_type);
            }
            Event::Transfer{from,to,value}=>{
                assert_eq!(value, &received.data.value);
                assert_eq!(EthEventType::Transfer { 
                    from: from.to_owned(),
                    to: to.to_owned()
                }, received.data.event_type);
            }
            Event::Deposit { to, value } => {
                assert_eq!(value, &received.data.value);
                assert_eq!(EthEventType::Deposit {
                    to: to.to_owned()
                }, received.data.event_type);
            },
            Event::Withdrawal { from, value } => {
                assert_eq!(value, &received.data.value);
                assert_eq!(EthEventType::Withdrawal {
                    from: from.to_owned()
                }, received.data.event_type);
            }
        }
    }

    // Assert that all events are persisted

    for event in received_events.iter() {
        let fetched_event = storage.fetch_all_events(None, event.id, 1).await.unwrap();
        assert_eq!(event, &fetched_event[0]);
    }
}
