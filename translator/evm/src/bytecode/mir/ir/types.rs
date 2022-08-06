use crate::bytecode::types::EthType;
use crate::U256;

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Clone, Hash, Copy)]
pub enum SType {
    U128,
    Bool,
}

pub type LocalIndex = u8;

#[derive(Debug)]
pub enum Value {
    U128(u128),
    Bool(bool),
}

impl From<U256> for Value {
    fn from(val: U256) -> Self {
        if val > U256::from(u128::MAX) {
            Value::U128(u128::MAX)
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
            Value::U128(_) => SType::U128,
            Value::Bool(_) => SType::Bool,
        }
    }
}

impl From<&EthType> for SType {
    fn from(tp: &EthType) -> Self {
        match tp {
            EthType::U256 => SType::U128,
            EthType::Bool => SType::Bool,
        }
    }
}
