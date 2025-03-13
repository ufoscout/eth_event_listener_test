use alloy::primitives::{Address, U256};

/// Ethereum event type.
/// This matches the events emitted by the IWETH9 contract
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Approval { from: Address, to: Address, value: U256 },
    Transfer { from: Address, to: Address, value: U256 },
    Deposit { to: Address, value: U256 },
    Withdrawal { from: Address, value: U256 },
}
