use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug)]
pub enum EntryType {
    #[serde(rename = "function")]
    Function,

    // A constructor is an optional function declared with the constructor keyword which is executed
    // upon contract creation, and where you can run contract initialisation code.
    // "constructor() {}"
    #[serde(rename = "constructor")]
    Constructor,

    // A contract can have at most one receive function, declared using
    // "receive() external payable { ... }"(without the function keyword).
    #[serde(rename = "receive")]
    Receive,

    // A constructor is an optional function declared with the constructor keyword which is executed
    // upon contract creation, and where you can run contract initialisation code.
    // "fallback () external [payable]"
    // "fallback (bytes calldata input) external [payable] returns (bytes memory output)"
    #[serde(rename = "fallback")]
    Fallback,
}

impl Default for EntryType {
    fn default() -> Self {
        EntryType::Function
    }
}
