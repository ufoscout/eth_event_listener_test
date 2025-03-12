use alloy::primitives::{Address, U256};

#[derive(Debug)]
pub enum Event {
    Approval { from: Address, to: Address, value: U256 },
    Transfer { from: Address, to: Address, value: U256 },
}
