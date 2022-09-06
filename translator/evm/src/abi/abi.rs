use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

use crate::abi::inc_ret_param::Param;
use crate::abi::types::StateMutability;
use anyhow::{Error, Result};
use itertools::Itertools;
use serde::{Deserialize, Deserializer};
use sha3::{Digest, Keccak256};

#[derive(Debug, Default)]
pub struct Abi {
    pub entries: Vec<Entry>,
}

impl<'de> serde::de::Deserialize<'de> for Abi {
    fn deserialize<D>(des: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let entries: Vec<Entry> = serde::Deserialize::deserialize(des)?;
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

#[derive(Debug, Deserialize, Eq, PartialEq)]
pub struct FunctionData {
    // The name of the function
    pub name: Option<String>,

    // An array of objects
    pub inputs: Option<Vec<Param>>,

    // an array of objects similar to inputs
    pub outputs: Option<Vec<Param>>,

    // State Mutability: "pure", "view", "nonpayable" or "payable"
    #[serde(alias = "stateMutability", default = "types::StateMutability::default")]
    pub state_mutability: StateMutability,
}

impl FunctionData {
    pub fn outputs(&self) -> Option<&Vec<Param>> {
        self.outputs.as_ref()
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
    use crate::abi::abi::{Abi, Entry, FunctionData};
    use crate::abi::call::ToCall;
    use crate::abi::inc_ret_param::types::ParamType;
    use crate::abi::inc_ret_param::Param;
    use crate::abi::types::StateMutability;
    use crate::abi::{Abi, Entry, FunctionData, Param};

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
        let _: Abi = serde_json::from_str(ABI_TEST).unwrap();
    }

    /// Encoding and decoding input/output
    ///
    /// https://docs.soliditylang.org/en/v0.8.0/abi-spec.html#examples
    /// ============================================================================================
    /// // SPDX-License-Identifier: GPL-3.0
    /// pragma solidity >=0.4.16 <0.9.0;
    ///
    /// contract Foo {
    ///    function bar(bytes3[2] memory) public pure {}
    ///    function baz(uint32 x, bool y) public pure returns (bool r) { r = x > 32 || y; }
    ///    function sam(bytes memory, bool, uint[] memory) public pure {}
    /// }
    /// ============================================================================================
    #[test]
    fn test_input_ecode() {
        let abi_str = r#"[
          {
            "inputs": [
              {
                "internalType": "bytes3[2]",
                "name": "",
                "type": "bytes3[2]"
              }
            ],
            "name": "bar",
            "outputs": [],
            "stateMutability": "pure",
            "type": "function"
          },
          {
            "inputs": [
              {
                "internalType": "uint32",
                "name": "x",
                "type": "uint32"
              },
              {
                "internalType": "bool",
                "name": "y",
                "type": "bool"
              }
            ],
            "name": "baz",
            "outputs": [
              {
                "internalType": "bool",
                "name": "r",
                "type": "bool"
              }
            ],
            "stateMutability": "pure",
            "type": "function"
          },
          {
            "inputs": [
              {
                "internalType": "bytes",
                "name": "",
                "type": "bytes"
              },
              {
                "internalType": "bool",
                "name": "",
                "type": "bool"
              },
              {
                "internalType": "uint256[]",
                "name": "",
                "type": "uint256[]"
              }
            ],
            "name": "sam",
            "outputs": [],
            "stateMutability": "pure",
            "type": "function"
          },
          {
            "inputs": [
              {
                "internalType": "uint256",
                "name": "",
                "type": "uint256"
              },
              {
                "internalType": "uint32[]",
                "name": "",
                "type": "uint32[]"
              },
              {
                "internalType": "bytes10",
                "name": "",
                "type": "bytes10"
              },
              {
                "internalType": "bytes",
                "name": "",
                "type": "bytes"
              }
            ],
            "name": "f",
            "outputs": [],
            "stateMutability": "pure",
            "type": "function"
          },
          {
            "inputs": [
              {
                "internalType": "uint256[][]",
                "name": "",
                "type": "uint256[][]"
              },
              {
                "internalType": "string[]",
                "name": "",
                "type": "string[]"
              }
            ],
            "name": "g",
            "outputs": [],
            "stateMutability": "pure",
            "type": "function"
          }
        ]"#;

        let abi: Abi = serde_json::from_str(abi_str).unwrap();
        // =========================================================================================
        // function baz(uint32 x, bool y)
        // =========================================================================================

        let entry_fn = abi.by_name("baz").unwrap();

        assert_eq!("0xcdcd77c0", &format!("0x{}", entry_fn.hash_hex()));

        let mut call_fn = entry_fn.try_call().unwrap();
        assert!(call_fn.clone().encode().is_err());

        call_fn.set_input(1, true).unwrap();
        assert!(call_fn.clone().encode().is_err());

        call_fn.set_input(0, 69u32).unwrap();

        let encode = hex::encode(call_fn.encode().unwrap());
        assert_eq!(
            "0xcdcd77c0\
            0000000000000000000000000000000000000000000000000000000000000045\
            0000000000000000000000000000000000000000000000000000000000000001",
            format!("0x{encode}")
        );

        // =========================================================================================
        // function bar(bytes3[2] memory)
        // =========================================================================================
        let entry_fn = abi.by_name("bar").unwrap();
        assert_eq!("0xfce353f6", &format!("0x{}", entry_fn.hash_hex()));

        let mut call_fn = entry_fn.try_call().unwrap();
        call_fn
            .set_input(0, ["abc".as_bytes(), "def".as_bytes()])
            .unwrap();
        let encode = hex::encode(call_fn.encode().unwrap());
        assert_eq!(
            "0xfce353f6\
            6162630000000000000000000000000000000000000000000000000000000000\
            6465660000000000000000000000000000000000000000000000000000000000",
            format!("0x{encode}")
        );

        // =========================================================================================
        // function sam(bytes memory, bool, uint[] memory)
        // sam("dave",true,[1,2,3])
        // =========================================================================================
        let entry_fn = abi.by_name("sam").unwrap();
        assert_eq!("0xa5643bf2", &format!("0x{}", entry_fn.hash_hex()));

        let mut call_fn = entry_fn.try_call().unwrap();
        call_fn.set_input(0, "dave".as_bytes()).unwrap();
        call_fn.set_input(1, true).unwrap();
        call_fn.set_input(2, [1usize, 2, 3]).unwrap();
        let encode = hex::encode(call_fn.encode().unwrap());

        assert_eq!(
            format!("0x{encode}"),
            "0xa5643bf2\
            0000000000000000000000000000000000000000000000000000000000000060\
            0000000000000000000000000000000000000000000000000000000000000001\
            00000000000000000000000000000000000000000000000000000000000000a0\
            0000000000000000000000000000000000000000000000000000000000000004\
            6461766500000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000003\
            0000000000000000000000000000000000000000000000000000000000000001\
            0000000000000000000000000000000000000000000000000000000000000002\
            0000000000000000000000000000000000000000000000000000000000000003",
        );

        // =========================================================================================
        // f(uint,uint32[],bytes10,bytes)
        // f(0x123, [0x456, 0x789], "1234567890", "Hello, world!")
        // =========================================================================================
        let entry_fn = abi.by_name("f").unwrap();
        assert_eq!("0x8be65246", &format!("0x{}", entry_fn.hash_hex()));

        let mut call_fn = entry_fn.try_call().unwrap();
        call_fn.set_input(0, 291usize).unwrap();
        call_fn.set_input(1, [1110u32, 1929u32]).unwrap();
        call_fn.set_input(2, "1234567890".as_bytes()).unwrap();
        call_fn.set_input(3, "Hello, world!".as_bytes()).unwrap();
        let encode = hex::encode(call_fn.encode().unwrap());

        assert_eq!(
            format!("0x{encode}"),
            "0x8be65246\
            0000000000000000000000000000000000000000000000000000000000000123\
            0000000000000000000000000000000000000000000000000000000000000080\
            3132333435363738393000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000e0\
            0000000000000000000000000000000000000000000000000000000000000002\
            0000000000000000000000000000000000000000000000000000000000000456\
            0000000000000000000000000000000000000000000000000000000000000789\
            000000000000000000000000000000000000000000000000000000000000000d\
            48656c6c6f2c20776f726c642100000000000000000000000000000000000000",
        );

        // =========================================================================================
        // g(uint[][],string[])
        // g([[1, 2], [3]], ["one", "two", "three"])
        // =========================================================================================
        let entry_fn = abi.by_name("g").unwrap();
        assert_eq!("0x2289b18c", &format!("0x{}", entry_fn.hash_hex()));

        let mut call_fn = entry_fn.try_call().unwrap();
        call_fn
            .set_input(0, vec![vec![1usize, 2], vec![3usize]])
            .unwrap();
        call_fn.set_input(1, ["one", "two", "three"]).unwrap();
        let encode = hex::encode(call_fn.encode().unwrap());

        assert_eq!(
            format!("0x{encode}"),
            "0x2289b18c\
            0000000000000000000000000000000000000000000000000000000000000040\
            0000000000000000000000000000000000000000000000000000000000000140\
            0000000000000000000000000000000000000000000000000000000000000002\
            0000000000000000000000000000000000000000000000000000000000000040\
            00000000000000000000000000000000000000000000000000000000000000a0\
            0000000000000000000000000000000000000000000000000000000000000002\
            0000000000000000000000000000000000000000000000000000000000000001\
            0000000000000000000000000000000000000000000000000000000000000002\
            0000000000000000000000000000000000000000000000000000000000000001\
            0000000000000000000000000000000000000000000000000000000000000003\
            0000000000000000000000000000000000000000000000000000000000000003\
            0000000000000000000000000000000000000000000000000000000000000060\
            00000000000000000000000000000000000000000000000000000000000000a0\
            00000000000000000000000000000000000000000000000000000000000000e0\
            0000000000000000000000000000000000000000000000000000000000000003\
            6f6e650000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000003\
            74776f0000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000005\
            7468726565000000000000000000000000000000000000000000000000000000",
        );
    }
}
