use std::fmt::{Display, Formatter};

use crate::bytecode::types::EthType;
use crate::U256;

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Clone, Hash, Copy)]
pub enum SType {
    Storage,
    Memory,
    Num,
    RawNum,
    Bool,
    Signer,
    Address,
    Bytes,
}

impl Display for SType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SType::Num => "num",
                SType::Bool => "bool",
                SType::Storage => "Storage",
                SType::Memory => "Memory",
                SType::Signer => "Signer",
                SType::Bytes => "vector<u8>",
                SType::Address => "address",
                SType::RawNum => "u128",
            }
        )
    }
}

pub type LocalIndex = u8;

#[derive(Debug, Clone)]
pub enum Value {
    Number(U256),
    Bool(bool),
}

impl From<U256> for Value {
    fn from(val: U256) -> Self {
        Value::Number(val)
    }
}

impl From<bool> for Value {
    fn from(val: bool) -> Self {
        Value::Bool(val)
    }
}

impl Value {
    pub fn s_type(&self) -> SType {
        match self {
            Value::Number(_) => SType::Num,
            Value::Bool(_) => SType::Bool,
        }
    }
}

impl SType {
    pub fn from_eth_type(eth_type: &EthType, u128_io: bool) -> Self {
        match eth_type {
            EthType::U256 => {
                if u128_io {
                    SType::RawNum
                } else {
                    SType::Num
                }
            }
            EthType::Bool => SType::Bool,
            EthType::Address => SType::Address,
            EthType::Bytes => SType::Bytes,
        }
    }
}
