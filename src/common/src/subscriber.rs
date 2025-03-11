use std::sync::{atomic::AtomicBool, Arc};

use alloy::{
    primitives::{address, Address, U256},
    providers::{Provider, ProviderBuilder, WsConnect},
    rpc::types::{BlockNumberOrTag, Filter},
    sol,
    sol_types::SolEvent,
};
use futures_util::stream::StreamExt;
use log::trace;
use tokio::sync::mpsc::UnboundedSender;

// Codegen from ABI file to interact with the contract.
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    IWETH9,
    "abi/IWETH9.json"
);

#[derive(Debug)]
pub enum Event {
    Approval {
        from: Address,
        to: Address,
        value: U256   
    },
    Transfer {
        from: Address,
        to: Address,
        value: U256
    }
}

pub async fn subscribe_to(rpc_url: &str, sender: UnboundedSender<Event>, run_until: Arc<AtomicBool>) -> anyhow::Result<()> {

    // Create a web socket provider.
    let ws = WsConnect::new(rpc_url);
    let provider = ProviderBuilder::new().on_ws(ws).await?;

    // Create a filter to watch for all WETH9 events.
    let weth9_token_address = address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");
    let filter = Filter::new()
        // By NOT specifying an `event` or `event_signature` we listen to ALL events of the
        // contract.
        .address(weth9_token_address)
        .from_block(BlockNumberOrTag::Latest);

    // Subscribe to logs.
    let sub = provider.subscribe_logs(&filter).await?;
    let mut stream = sub.into_stream().take_while(|_x| async {
        run_until.load(std::sync::atomic::Ordering::Relaxed)
    }).boxed();

        while let Some(log) = stream.next().await {
            // Match on topic 0, the hash of the signature of the event.
            match log.topic0() {
                // Match the `Approval(address,address,uint256)` event.
                Some(&IWETH9::Approval::SIGNATURE_HASH) => {
                    let IWETH9::Approval { src, guy, wad } = log.log_decode()?.inner.data;
                    trace!("Received event from subscription: Approval from {src} to {guy} of value {wad}");
                    sender.send(Event::Approval { from: src, to: guy, value: wad })?;
                }
                // Match the `Transfer(address,address,uint256)` event.
                Some(&IWETH9::Transfer::SIGNATURE_HASH) => {
                    let IWETH9::Transfer { src, dst, wad } = log.log_decode()?.inner.data;
                    trace!("Received event from subscription: Transfer from {src} to {dst} of value {wad}");
                    sender.send(Event::Transfer { from: src, to: dst, value: wad })?;
                }
                // WETH9's `Deposit(address,uint256)` and `Withdrawal(address,uint256)` events are not
                // handled here.
                _ => (),
            }
        }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::config::test::get_settings;

    use super::*;

    #[tokio::test]
    async fn test() {
        // Arrange
        let settings = get_settings();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let run_until = Arc::new(AtomicBool::new(true));

        // Act
        let run_until_clone = run_until.clone();
        tokio::spawn(async move {
                subscribe_to(&settings.eth_node.wss_url, tx, run_until_clone).await
                .expect("Failed to run main");
        });

        // Assert
        // wait for 5 events
        for _ in 0..5 {
            let event = rx.recv().await.expect("Failed to receive event");
            println!("Received event: {event:?}");
        }

        run_until.store(false, std::sync::atomic::Ordering::Relaxed);

    }   
}