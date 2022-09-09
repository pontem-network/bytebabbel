use crate::bytecode::types::EthType;
use crate::U256;
use std::fmt::{Display, Formatter};

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Clone, Hash, Copy)]
pub enum SType {
    Storage,
    Memory,
    Num,
    Bool,
    Address,
    Bytes,
}

impl Display for SType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SType::Num => "u128",
                SType::Bool => "bool",
                SType::Storage => "Storage",
                SType::Memory => "Memory",
                SType::Address => "Address",
                SType::Bytes => "vector<u8>",
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

impl From<&EthType> for SType {
    fn from(tp: &EthType) -> Self {
        match tp {
            EthType::U256 => SType::Num,
            EthType::Bool => SType::Bool,
            EthType::Address => SType::Address,
        }
    }
}
