use alloy::primitives::{Address, U256};
use c3p0::Model;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumDiscriminants};

pub type EthEventModel = Model<u64, EthEventData>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EthEventData {
    pub value: U256,
    pub event_type: EthEventType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, AsRefStr, Display, EnumDiscriminants)]
#[strum_discriminants(derive(Serialize, Deserialize, AsRefStr, Display))]
#[serde(tag = "type")]
pub enum EthEventType {
    Approve {from: Address, to: Address},
    Transfer {from: Address, to: Address},
    Deposit {to: Address},
    Withdrawal {from: Address},
}
