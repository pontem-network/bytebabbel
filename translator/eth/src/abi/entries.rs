use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

use crate::abi::inc_ret_param::Param;
use crate::abi::types::StateMutability;
use anyhow::{Error, Result};
use itertools::Itertools;
use serde::{Deserialize, Deserializer};
use sha3::{Digest, Keccak256};

#[derive(Debug, Default, Clone)]
pub struct AbiEntries {
    pub entries: Vec<Entry>,
}

impl AbiEntries {
    pub fn by_name(&self, name: &str) -> Option<&Entry> {
        self.entries
            .iter()
            .find(|item| item.name().as_deref() == Some(name))
    }

    pub fn constructor(&self) -> Option<&Entry> {
        self.entries.iter().find(|item| item.is_constructor())
    }
}

impl<'de> serde::de::Deserialize<'de> for AbiEntries {
    fn deserialize<D>(des: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let entries: Vec<Entry> = serde::Deserialize::deserialize(des)?;
        Ok(AbiEntries { entries })
    }
}

impl TryFrom<&str> for AbiEntries {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let result = serde_json::from_str(value)?;
        Ok(result)
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq, Clone)]
#[serde(tag = "type")]
pub enum Entry {
    // error InsufficientBalance(uint256 available, uint256 required);
    #[serde(rename = "error")]
    Error { name: String, inputs: Vec<Param> },

    // event Received(address, uint);
    #[serde(rename = "event")]
    Event {
        name: String,
        inputs: Vec<Param>,
        // true if the event was declared as anonymous.
        anonymous: Option<bool>,
    },

    // #############################################################################################
    // Functions
    // #############################################################################################
    #[serde(rename = "function")]
    Function(FunctionData),

    // A constructor is an optional function declared with the constructor keyword which is executed
    // upon contract creation, and where you can run contract initialisation code.
    // "constructor() {}"
    #[serde(rename = "constructor")]
    Constructor(FunctionData),

    // A contract can have at most one receive function, declared using
    // "receive() external payable { ... }"(without the function keyword).
    #[serde(rename = "receive")]
    Receive(FunctionData),

    // A constructor is an optional function declared with the constructor keyword which is executed
    // upon contract creation, and where you can run contract initialisation code.
    // "fallback () external [payable]"
    // "fallback (bytes calldata input) external [payable] returns (bytes memory output)"
    #[serde(rename = "fallback")]
    Fallback(FunctionData),
}

impl Entry {
    pub fn is_constructor(&self) -> bool {
        matches!(self, Entry::Constructor(_))
    }

    pub fn is_function(&self) -> bool {
        matches!(
            self,
            Entry::Function(_) | Entry::Constructor(_) | Entry::Receive(_) | Entry::Fallback(_)
        )
    }

    pub fn function_data(&self) -> Option<&FunctionData> {
        match self {
            Entry::Function(data)
            | Entry::Constructor(data)
            | Entry::Receive(data)
            | Entry::Fallback(data) => Some(data),
            _ => None,
        }
    }

    pub fn signature(&self) -> String {
        let types = self
            .inputs()
            .map(|inp| inp.iter().map(|d| d.tp.to_string()).join(","))
            .unwrap_or_default();
        format!(
            "{name}({types})",
            name = self.name().as_deref().unwrap_or("anonymous")
        )
    }

    pub fn name(&self) -> Option<String> {
        match self {
            Entry::Function(data)
            | Entry::Constructor(data)
            | Entry::Receive(data)
            | Entry::Fallback(data) => data.name.clone(),
            Entry::Error { name, .. } | Entry::Event { name, .. } => Some(name.clone()),
        }
    }

    pub fn inputs(&self) -> Option<&Vec<Param>> {
        match self {
            Entry::Function(data)
            | Entry::Constructor(data)
            | Entry::Receive(data)
            | Entry::Fallback(data) => data.inputs.as_ref(),
            Entry::Error { inputs, .. } | Entry::Event { inputs, .. } => Some(inputs),
        }
    }

    pub fn outputs(&self) -> Option<&Vec<Param>> {
        match self {
            Entry::Function(data)
            | Entry::Constructor(data)
            | Entry::Receive(data)
            | Entry::Fallback(data) => data.outputs.as_ref(),
            Entry::Error { .. } | Entry::Event { .. } => None,
        }
    }

    /// hash of the function in hexadecimal format without the prefix "0x"
    pub fn hash_hex(&self) -> String {
        let hash: FunHash = self.into();
        hex::encode(hash.0)
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq, Clone)]
pub struct FunctionData {
    // The name of the function
    pub name: Option<String>,

    // An array of objects
    pub inputs: Option<Vec<Param>>,

    // an array of objects similar to inputs
    pub outputs: Option<Vec<Param>>,

    // State Mutability: "pure", "view", "nonpayable" or "payable"
    #[serde(alias = "stateMutability", default = "StateMutability::default")]
    pub state_mutability: StateMutability,
}

impl FunctionData {
    pub fn outputs(&self) -> Option<&Vec<Param>> {
        self.outputs.as_ref()
    }
}

pub const FUN_HASH_LEN: usize = 4;

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Default)]
pub struct FunHash([u8; FUN_HASH_LEN]);

impl AsRef<[u8; FUN_HASH_LEN]> for FunHash {
    fn as_ref(&self) -> &[u8; FUN_HASH_LEN] {
        &self.0
    }
}

impl From<&Entry> for FunHash {
    fn from(entry: &Entry) -> Self {
        let mut result = [0u8; FUN_HASH_LEN];
        result.copy_from_slice(&Keccak256::digest(&entry.signature())[..FUN_HASH_LEN]);
        FunHash(result)
    }
}

impl Debug for FunHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl Display for FunHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// @todo
#[cfg(test)]
mod tests {
    use crate::abi::entries::{AbiEntries, Entry, FunctionData};
    use crate::abi::inc_ret_param::types::ParamType;
    use crate::abi::inc_ret_param::Param;
    use crate::abi::types::StateMutability;

    #[test]
    fn test_entry_deserialize_error() {
        // error InsufficientBalance(uint256 available, uint256 required);
        let content = r#"{
            "inputs": [
                {
                    "internalType": "uint256",
                    "name": "available",
                    "type": "uint256"
                },
                {
                    "internalType": "uint256",
                    "name": "required",
                    "type": "uint256"
                }
            ],
            "name": "InsufficientBalance",
            "type": "error"
        }"#;
        let result: Entry = serde_json::from_str(content).unwrap();

        assert_eq!(
            Entry::Error {
                name: "InsufficientBalance".to_string(),
                inputs: vec![
                    Param {
                        name: "available".to_string(),
                        tp: ParamType::UInt(256),
                        internal_type: Some(ParamType::UInt(256)),
                        components: None,
                        indexed: None
                    },
                    Param {
                        name: "required".to_string(),
                        tp: ParamType::UInt(256),
                        internal_type: Some(ParamType::UInt(256)),
                        components: None,
                        indexed: None
                    },
                ]
            },
            result
        );
    }

    #[test]
    fn test_entry_deserialize_event() {
        // event Received(address, uint);
        let mut content = r#" {
            "anonymous": false,
            "inputs": [
                {
                    "indexed": false,
                    "internalType": "address",
                    "name": "",
                    "type": "address"
                },
                {
                    "indexed": false,
                    "internalType": "uint256",
                    "name": "",
                    "type": "uint256"
                }
            ],
            "name": "Received",
            "type": "event"
        }"#;
        let mut result: Entry = serde_json::from_str(content).unwrap();
        assert_eq!(
            Entry::Event {
                name: "Received".to_string(),
                inputs: vec![
                    Param {
                        name: "".to_string(),
                        tp: ParamType::Address,
                        internal_type: Some(ParamType::Address),
                        components: None,
                        indexed: Some(false)
                    },
                    Param {
                        name: "".to_string(),
                        tp: ParamType::UInt(256),
                        internal_type: Some(ParamType::UInt(256)),
                        components: None,
                        indexed: Some(false)
                    }
                ],
                anonymous: Some(false)
            },
            result
        );

        // event Deposit(
        //     address indexed from,
        //     bytes32 indexed id,
        //     uint value
        // );
        content = r#"{
            "anonymous": false,
            "inputs": [
                {
                    "indexed": true,
                    "internalType": "address",
                    "name": "from",
                    "type": "address"
                },
                {
                    "indexed": true,
                    "internalType": "bytes32",
                    "name": "id",
                    "type": "bytes32"
                },
                {
                    "indexed": false,
                    "internalType": "uint256",
                    "name": "value",
                    "type": "uint256"
                }
            ],
            "name": "Deposit",
            "type": "event"
        }"#;
        result = serde_json::from_str(content).unwrap();
        assert_eq!(
            Entry::Event {
                name: "Deposit".to_string(),
                inputs: vec![
                    Param {
                        name: "from".to_string(),
                        tp: ParamType::Address,
                        internal_type: Some(ParamType::Address),
                        components: None,
                        indexed: Some(true)
                    },
                    Param {
                        name: "id".to_string(),
                        tp: ParamType::Byte(32),
                        internal_type: Some(ParamType::Byte(32)),
                        components: None,
                        indexed: Some(true)
                    },
                    Param {
                        name: "value".to_string(),
                        tp: ParamType::UInt(256),
                        internal_type: Some(ParamType::UInt(256)),
                        components: None,
                        indexed: Some(false)
                    }
                ],
                anonymous: Some(false)
            },
            result
        );
    }

    #[test]
    fn test_entry_deserialize_constructor() {
        // constructor(bytes32 name_) { }
        let content = r#"{
            "inputs": [
                {
                    "internalType": "bytes32",
                    "name": "name_",
                    "type": "bytes32"
                }
            ],
            "stateMutability": "nonpayable",
            "type": "constructor"
        }"#;
        let result: Entry = serde_json::from_str(content).unwrap();

        assert_eq!(
            Entry::Constructor(FunctionData {
                name: None,
                inputs: Some(vec![Param {
                    name: "name_".to_string(),
                    tp: ParamType::Byte(32),
                    internal_type: Some(ParamType::Byte(32)),
                    components: None,
                    indexed: None,
                }]),
                state_mutability: StateMutability::Nonpayable,
                outputs: None
            }),
            result
        );
    }

    #[test]
    fn test_entry_deserialize_function() {
        // pure
        // function transfer(address newOwner) public { }
        let mut content = r#"{
            "inputs": [
                {
                    "internalType": "address",
                    "name": "newOwner",
                    "type": "address"
                }
            ],
            "name": "transfer",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        }"#;
        let mut fun: Entry = serde_json::from_str(content).unwrap();

        assert_eq!(
            Entry::Function(FunctionData {
                name: Some("transfer".to_string()),
                inputs: Some(vec![Param {
                    name: "newOwner".to_string(),
                    tp: ParamType::Address,
                    internal_type: Some(ParamType::Address),
                    components: None,
                    indexed: None,
                }]),
                outputs: Some(Vec::new()),
                state_mutability: StateMutability::Nonpayable
            }),
            fun
        );

        // view
        // function f3() public view returns (uint) { return 2; }
        content = r#"{
            "inputs": [],
            "name": "f3",
            "outputs": [
                {
                    "internalType": "uint256",
                    "name": "",
                    "type": "uint256"
                }
            ],
            "stateMutability": "view",
            "type": "function"
        }"#;
        fun = serde_json::from_str(content).unwrap();
        assert_eq!(
            Entry::Function(FunctionData {
                name: Some("f3".to_string()),
                inputs: Some(vec![]),
                outputs: Some(vec![Param {
                    name: "".to_string(),
                    tp: ParamType::UInt(256),
                    internal_type: Some(ParamType::UInt(256)),
                    components: None,
                    indexed: None
                }]),
                state_mutability: StateMutability::View
            }),
            fun
        );

        // nonpayable
        // function readData() public { }
        content = r#" {
            "inputs": [],
            "name": "readData",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        }"#;
        fun = serde_json::from_str(content).unwrap();
        assert_eq!(
            Entry::Function(FunctionData {
                name: Some("readData".to_string()),
                inputs: Some(vec![]),
                outputs: Some(vec![]),
                state_mutability: StateMutability::Nonpayable
            }),
            fun
        );
    }

    #[test]
    fn test_entry_deserialize_fallback() {
        // fallback() external { x = 1; }
        let content = r#"{
            "stateMutability": "nonpayable",
            "type": "fallback"
        }"#;
        let fun: Entry = serde_json::from_str(content).unwrap();

        assert_eq!(
            Entry::Fallback(FunctionData {
                name: None,
                inputs: None,
                outputs: None,
                state_mutability: StateMutability::Nonpayable
            }),
            fun
        );
    }

    #[test]
    fn test_entry_deserialize_receive() {
        // receive() external payable { emit Received(msg.sender, msg.value); }
        let content = r#"{
            "stateMutability": "payable",
            "type": "receive"
        }"#;
        let fun: Entry = serde_json::from_str(content).unwrap();

        assert_eq!(
            Entry::Receive(FunctionData {
                name: None,
                inputs: None,
                outputs: None,
                state_mutability: StateMutability::Payable
            }),
            fun
        );
    }

    #[test]
    fn test_deserialize_nabi() {
        const ABI_TEST: &str = r#"[{
            "type":"error",
            "inputs": [{"name":"available","type":"uint256"},{"name":"required","type":"uint256"}],
            "name":"InsufficientBalance"
        }, {
            "type":"event",
            "inputs": [{"name":"a","type":"uint256","indexed":true},{"name":"b","type":"bytes32","indexed":false}],
            "name":"Event"
        }, {
            "type":"event",
            "inputs": [{"name":"a","type":"uint256","indexed":true},{"name":"b","type":"bytes32","indexed":false}],
            "name":"Event2"
        }, {
            "type":"function",
            "inputs": [{"name":"a","type":"uint256"}],
            "name":"foo",
            "outputs": []
        }]"#;
        let _: AbiEntries = serde_json::from_str(ABI_TEST).unwrap();
    }
}
