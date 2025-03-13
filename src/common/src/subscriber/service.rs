use std::sync::{Arc, atomic::AtomicBool};

use alloy::{
    primitives::Address,
    providers::{Provider, ProviderBuilder, WsConnect},
    rpc::types::{BlockNumberOrTag, Filter, Log},
    sol,
    sol_types::SolEvent,
};
use futures_util::{Stream, stream::StreamExt};
use log::*;
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle, time::timeout};

use super::model::Event;

// Codegen from ABI file to interact with the contract.
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    IWETH9,
    "resources/abi/IWETH9.json"
);

/// Service for subscribing to Ethereum events
pub struct SubscriberService {
    rpc_url: String,
    timeout_seconds: u64,
    token_address: Address,
}

impl SubscriberService {

    /// Creates a new instance of `SubscriberService`.
    ///
    /// `rpc_url` should be the URL of the Ethereum node WebSocket endpoint to connect to
    /// `timeout_seconds` is the time in seconds to wait for an event before attempting a reconnection
    /// `token_address` is the address of the ERC20 token to subscribe to events of.
    pub fn new(rpc_url: String, timeout_seconds: u64, token_address: Address) -> Self {
        Self { rpc_url, timeout_seconds, token_address }
    }

    
    /// Subscribes to Ethereum events for the specific token address and sends them to the provided channel.
    ///
    /// This function spawns a new tokio task that listens for events then returns a `JoinHandle` for the spawned task.
    ///
    /// # Arguments
    ///
    /// * `sender` - An unbounded sender to transmit decoded Ethereum events.
    /// * `run_until` - An atomic boolean that controls the lifetime of the subscription. The task will exit when it is set to `false`.
    ///
    /// # Errors
    ///
    /// Returns an error if the subscription to the Ethereum node fails.
    pub async fn subscribe_to(
        &self,
        sender: UnboundedSender<Event>,
        run_until: Arc<AtomicBool>,
    ) -> anyhow::Result<JoinHandle<()>> {

        info!("SubscriberService - Subscribing to events for token {}", self.token_address);

        let filter = Filter::new().address(self.token_address).from_block(BlockNumberOrTag::Latest);

        let rpc_url = self.rpc_url.clone();
        let timeout_seconds = std::time::Duration::from_secs(self.timeout_seconds);

        let handle = tokio::spawn(async move {
            let (mut _provider, mut stream) = new_subscription(&rpc_url, &filter, &run_until).await.unwrap();

            loop {
                let result = timeout(timeout_seconds, stream.next()).await;
                match result {
                    Ok(Some(log)) => match decode_log(log, &sender) {
                        Ok(()) => debug!("Log processed successfully"),
                        Err(err) => error!("Error while processing received log: {err:?}"),
                    },
                    Ok(None) => {
                        warn!("WS connection was closed. Reconnecting...");
                        match new_subscription(&rpc_url, &filter, &run_until)
                            .await {
                                Ok((new_provider, new_stream)) => {
                                    _provider = new_provider;
                                    stream = new_stream;
                                },
                                Err(err) => {
                                    error!("Failed to reconnect. The service will exit. Error: {err:?}");
                                    break;
                            }
                        }
                    }
                    Err(_err) => {
                        warn!(
                            "WS connection not received any event in {} seconds. Reconnecting...",
                            timeout_seconds.as_secs()
                        );
                        match new_subscription(&rpc_url, &filter, &run_until)
                        .await {
                            Ok((new_provider, new_stream)) => {
                                _provider = new_provider;
                                stream = new_stream;
                            },
                            Err(err) => {
                                error!("Failed to reconnect. The service will exit. Error: {err:?}");
                                break;
                        }
                    }
                    }
                }
            }
        });

        Ok(handle)
    }
}


/// Establishes a new subscription to the Ethereum node for the given `rpc_url`.
async fn new_subscription(
    rpc_url: &str,
    filter: &Filter,
    run_until: &AtomicBool,
) -> anyhow::Result<(impl Provider, impl Stream<Item = Log>)> {
    let ws = WsConnect::new(rpc_url);
    let provider = ProviderBuilder::new().on_ws(ws).await?;

    let sub = provider.subscribe_logs(filter).await?;

    let stream =
        sub.into_stream().take_while(|_x| async { run_until.load(std::sync::atomic::Ordering::Relaxed) }).boxed();

    Ok((provider, stream))
}

/// Decodes and sends an Ethereum event to the provided channel.
fn decode_log(log: Log, sender: &UnboundedSender<Event>) -> anyhow::Result<()> {

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

        // Match the `Deposit(address,uint256)` event.
        Some(&IWETH9::Deposit::SIGNATURE_HASH) => {
            let IWETH9::Deposit { dst, wad } = log.log_decode()?.inner.data;
            debug!("Received event from subscription: Deposit to {dst} of value {wad}");
            sender.send(Event::Deposit { to: dst, value: wad })?;
        }

        // Match the `Withdrawal(address,uint256)` event.
        Some(&IWETH9::Withdrawal::SIGNATURE_HASH) => {
            let IWETH9::Withdrawal { src, wad } = log.log_decode()?.inner.data;
            debug!("Received event from subscription: Withdrawal from {src} of value {wad}");
            sender.send(Event::Withdrawal { from: src, value: wad })?;
        }

        // Unknown event
        event => {
            warn!("Received unknown event: {event:?}");
        }
    };
    Ok(())
}
