use c3p0::Model;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display};

pub type EthEventModel = Model<u64, EthEventData>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EthEventData {
    pub from: String,
    pub to: String,
    pub event_type: EthEventType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, AsRefStr, Display)]
pub enum EthEventType {
    Approve,
    Transfer,
}