use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub enum StateMutability {
    // specified to not modify the blockchain state
    #[serde(rename = "view", alias = "constant")]
    View,

    // specified to not read blockchain state
    #[serde(rename = "pure")]
    Pure,

    // function does not accept Ether - the default
    #[serde(rename = "nonpayable")]
    Nonpayable,

    // function accepts Ether
    #[serde(rename = "payable")]
    Payable,
}

impl Default for StateMutability {
    fn default() -> Self {
        StateMutability::Nonpayable
    }
}
