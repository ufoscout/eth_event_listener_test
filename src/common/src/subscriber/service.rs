use std::sync::{Arc, atomic::AtomicBool};

use alloy::{
    primitives::Address,
    providers::{Provider, ProviderBuilder, WsConnect},
    rpc::types::{BlockNumberOrTag, Filter, Log},
    sol,
    sol_types::SolEvent,
};
use futures_util::stream::StreamExt;
use log::*;
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};

use super::model::Event;

// Codegen from ABI file to interact with the contract.
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    IWETH9,
    "resources/abi/IWETH9.json"
);

pub struct SubscriberService {
    rpc_url: String,
    token_address: Address,
}

impl SubscriberService {
    pub fn new(rpc_url: String, token_address: Address) -> Self {
        Self { rpc_url, token_address }
    }

    pub async fn subscribe_to(
        &self,
        sender: UnboundedSender<Event>,
        run_until: Arc<AtomicBool>,
    ) -> anyhow::Result<JoinHandle<()>> {
        // Create a web socket provider.
        let ws = WsConnect::new(&self.rpc_url);
        let filter = Filter::new()
            // By NOT specifying an `event` or `event_signature` we listen to ALL events of the
            // contract.
            .address(self.token_address)
            .from_block(BlockNumberOrTag::Latest);

        let handle = tokio::spawn(async move {
            let provider = ProviderBuilder::new().on_ws(ws).await.unwrap();

            // Subscribe to logs.
            let sub = provider.subscribe_logs(&filter).await.unwrap();

            let mut stream = sub
                .into_stream()
                .take_while(|_x| async { run_until.load(std::sync::atomic::Ordering::Relaxed) })
                .boxed();

            while let Some(log) = stream.next().await {
                match decode_log(log, &sender) {
                    Ok(()) => debug!("Log processed successfully"),
                    Err(err) => error!("Error while processing received log: {err:?}"),
                }
            }
        });

        Ok(handle)
    }
}

fn decode_log(log: Log, sender: &UnboundedSender<Event>) -> anyhow::Result<()> {
    // Match on topic 0, the hash of the signature of the event.
    match log.topic0() {
        // Match the `Approval(address,address,uint256)` event.
        Some(&IWETH9::Approval::SIGNATURE_HASH) => {
            let IWETH9::Approval { src, guy, wad } = log.log_decode()?.inner.data;
            debug!("Received event from subscription: Approval from {src} to {guy} of value {wad}");
            sender.send(Event::Approval { from: src, to: guy, value: wad })?;
        }
        // Match the `Transfer(address,address,uint256)` event.
        Some(&IWETH9::Transfer::SIGNATURE_HASH) => {
            let IWETH9::Transfer { src, dst, wad } = log.log_decode()?.inner.data;
            debug!("Received event from subscription: Transfer from {src} to {dst} of value {wad}");
            sender.send(Event::Transfer { from: src, to: dst, value: wad })?;
        }
        // WETH9's `Deposit(address,uint256)` and `Withdrawal(address,uint256)` events are not
        // handled here.
        _ => (),
    };
    Ok(())
}
