use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

use anyhow::Error;
use itertools::Itertools;
use serde::{Deserialize, Deserializer};
use sha3::{Digest, Keccak256};

pub mod inc_ret_param;
pub mod types;

use inc_ret_param::Param;

#[derive(Debug, Default)]
pub struct Abi {
    entries: HashMap<FunHash, Entry>,
}

impl Abi {
    pub fn fun_hashes(&self) -> impl Iterator<Item = FunHash> + '_ {
        self.entries
            .iter()
            .filter(|(_, abi)| abi.is_function())
            .map(|(h, _)| *h)
    }

    pub fn entry(&self, hash: &FunHash) -> Option<&Entry> {
        self.entries.get(hash)
    }
}

impl<'de> serde::de::Deserialize<'de> for Abi {
    fn deserialize<D>(des: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let container: Vec<Entry> = serde::Deserialize::deserialize(des)?;
        let entries = container
            .into_iter()
            .map(|e| (FunHash::from(&e), e))
            .collect();
        Ok(Abi { entries })
    }
}

impl TryFrom<&str> for Abi {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let result = serde_json::from_str(value)?;
        Ok(result)
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(tag = "type")]
pub enum Entry {
    #[serde(rename = "error")]
    Error { name: String, inputs: Vec<Param> },

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

    // @todo tests

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
        let types = self.inputs().iter().map(|d| d.tp.to_string()).join(",");
        let name = self.name();
        format!("{name}({types})")
    }

    pub fn name(&self) -> String {
        match self {
            Entry::Function(data)
            | Entry::Constructor(data)
            | Entry::Receive(data)
            | Entry::Fallback(data) => data.name.clone(),
            Entry::Error { name, .. } | Entry::Event { name, .. } => name.clone(),
        }
    }

    pub fn inputs(&self) -> &Vec<Param> {
        match self {
            Entry::Function(data)
            | Entry::Constructor(data)
            | Entry::Receive(data)
            | Entry::Fallback(data) => &data.inputs,
            Entry::Error { inputs, .. } | Entry::Event { inputs, .. } => inputs,
        }
    }

    pub fn outputs(&self) -> Option<&Vec<Param>> {
        match self {
            Entry::Function(data)
            | Entry::Constructor(data)
            | Entry::Receive(data)
            | Entry::Fallback(data) => Some(&data.outputs),
            Entry::Error { .. } | Entry::Event { .. } => None,
        }
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct FunctionData {
    // The name of the function
    pub name: String,

    // An array of objects
    pub inputs: Vec<Param>,

    // an array of objects similar to inputs
    pub outputs: Vec<Param>,

    // State Mutability: "pure", "view", "nonpayable" or "payable"
    #[serde(alias = "stateMutability", default = "types::StateMutability::default")]
    pub state_mutability: types::StateMutability,
}

impl FunctionData {
    pub fn outputs(&self) -> &[Param] {
        &self.outputs
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Default)]
pub struct FunHash([u8; 4]);

impl AsRef<[u8; 4]> for FunHash {
    fn as_ref(&self) -> &[u8; 4] {
        &self.0
    }
}

impl From<&Entry> for FunHash {
    fn from(entry: &Entry) -> Self {
        let mut result = [0u8; 4];
        result.copy_from_slice(&Keccak256::digest(&entry.signature())[..4]);
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

#[cfg(test)]
mod tests {
    use crate::abi::inc_ret_param::types::ParamType;
    use crate::abi::types::StateMutability;
    use crate::abi::{Abi, Entry, FunctionData, Param};

    #[test]
    fn test_deserialize_type_error() {
        let content = r#"{
            "type":"error",
            "inputs": [{"name":"available","type":"uint256"},{"name":"required","type":"bool"}],
            "name":"InsufficientBalance"
        }"#;

        let error: Entry = serde_json::from_str(content).unwrap();

        assert_eq!(
            Entry::Error {
                name: "InsufficientBalance".to_string(),
                inputs: vec![
                    Param {
                        name: "available".to_string(),
                        tp: ParamType::Uint(256),
                        components: None,
                        indexed: None
                    },
                    Param {
                        name: "required".to_string(),
                        tp: ParamType::Bool,
                        components: None,
                        indexed: None
                    },
                ]
            },
            error
        );
    }

    #[test]
    fn test_deserialize_type_function() {
        let content = r#"{
            "type":"function",
            "inputs": [{"name":"a","type":"uint256"}],
            "name":"foo",
            "outputs": []
        }"#;

        let fun: Entry = serde_json::from_str(content).unwrap();

        assert_eq!(
            Entry::Function(FunctionData {
                name: "foo".to_string(),
                inputs: vec![Param {
                    name: "a".to_string(),
                    tp: ParamType::Uint(256),
                    components: None,
                    indexed: None
                }],
                outputs: vec![],
                state_mutability: StateMutability::Nonpayable
            }),
            fun
        );
    }

    #[test]
    fn test_deserialize_type_event() {
        let content = r#"{
            "type":"event",
            "inputs": [{"name":"a","type":"uint256","indexed":true},{"name":"b","type":"bytes32","indexed":false}],
            "name":"Event2"
        }"#;

        let event: Entry = serde_json::from_str(content).unwrap();

        assert_eq!(
            Entry::Event {
                name: "Event2".to_string(),
                inputs: vec![
                    Param {
                        name: "a".to_string(),
                        tp: ParamType::Uint(256),
                        components: None,
                        indexed: Some(true)
                    },
                    Param {
                        name: "b".to_string(),
                        tp: ParamType::Byte(32),
                        components: None,
                        indexed: Some(false)
                    }
                ],
                anonymous: None
            },
            event
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
        let _: Abi = serde_json::from_str(ABI_TEST).unwrap();
    }
}
