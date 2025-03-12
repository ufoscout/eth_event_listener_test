use std::{
    str::FromStr,
    sync::{Arc, atomic::AtomicBool},
};

use alloy::primitives::Address;
use common::subscriber::service::SubscriberService;

use crate::get_settings;

#[tokio::test]
async fn test_subscription_to_remote_node() {
    // Arrange
    let settings = get_settings();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let run_until = Arc::new(AtomicBool::new(true));

    let subscriber =
        SubscriberService::new(settings.eth_node.wss_url, Address::from_str(&settings.eth_node.token_address).unwrap());

    // Act
    let run_until_clone = run_until.clone();
    subscriber.subscribe_to(tx, run_until_clone).await.expect("Failed to subscribe");

    // Assert
    // wait for 5 events
    for _ in 0..5 {
        let event = rx.recv().await.unwrap();
        println!("Received event: {event:?}");
    }

    run_until.store(false, std::sync::atomic::Ordering::Relaxed);
}
