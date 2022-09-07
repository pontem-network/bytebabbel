use crate::bytecode::types::EthType;
use crate::U256;
use std::fmt::{Display, Formatter};

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Clone, Hash, Copy)]
pub enum SType {
    Storage,
    Memory,
    Number,
    Bool,
    Address,
}

impl Display for SType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SType::Number => "u128",
                SType::Bool => "bool",
                SType::Storage => "Storage",
                SType::Memory => "Memory",
                SType::Address => "Address",
            }
        )
    }
}

pub type LocalIndex = u8;

#[derive(Debug, Clone)]
pub enum Value {
    U128(u128),
    Bool(bool),
}

impl From<U256> for Value {
    fn from(val: U256) -> Self {
        if val > U256::from(u128::MAX) {
            Value::U128(val.low_u128())
        } else {
            Value::U128(val.as_u128())
        }
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
            Value::U128(_) => SType::Number,
            Value::Bool(_) => SType::Bool,
        }
    }
}

impl From<&EthType> for SType {
    fn from(tp: &EthType) -> Self {
        match tp {
            EthType::U256 => SType::Number,
            EthType::Bool => SType::Bool,
            EthType::Address => SType::Address,
        }
    }
}
