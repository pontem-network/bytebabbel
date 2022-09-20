#![allow(clippy::assign_op_pattern)]
#![allow(clippy::ptr_offset_with_cast)]

use crate::abi::entries::{FunHash, FRAME_SIZE, FUN_HASH_LEN};
use crate::abi::inc_ret_param::types::ParamType;
use crate::abi::inc_ret_param::Param as AbiType;
use anyhow::{bail, Error};
use primitive_types::U256;
use std::fmt::{Display, Formatter};

#[derive(Default, Clone, Debug)]
pub struct Function {
    pub hash: FunHash,
    pub name: String,
    pub eth_input: Vec<EthType>,
    pub native_input: Vec<EthType>,
    pub eth_output: Vec<EthType>,
    pub native_output: Vec<EthType>,
}

impl Function {
    pub fn call_data_size(&self) -> U256 {
        U256::from(self.native_input.len() * FRAME_SIZE + FUN_HASH_LEN)
    }

    pub fn hash(&self) -> FunHash {
        self.hash
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({:?}) -> ({:?})",
            self.name, self.eth_input, self.eth_output
        )
    }
}

#[derive(Debug)]
pub struct Constructor {
    pub move_input: Vec<EthType>,
    pub eth_input: Vec<EthType>,
}

impl Default for Constructor {
    fn default() -> Self {
        Constructor {
            move_input: vec![EthType::Address],
            eth_input: vec![],
        }
    }
}

impl Display for Constructor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "constructor({:?})", self.move_input)
    }
}

impl From<&Constructor> for Function {
    fn from(c: &Constructor) -> Self {
        Function {
            hash: Default::default(),
            name: "constructor".to_string(),
            eth_input: c.move_input.clone(),
            native_input: c.eth_input.clone(),
            eth_output: vec![],
            native_output: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub enum EthType {
    U256,
    Bool,
    Address,
    Bytes,
}

impl<'a> TryFrom<&'a AbiType> for EthType {
    type Error = Error;

    fn try_from(value: &'a AbiType) -> Result<Self, Self::Error> {
        Ok(match value.tp {
            ParamType::Bool => EthType::Bool,
            ParamType::UInt(_) | ParamType::Int(_) => EthType::U256,
            ParamType::String => EthType::Bytes,
            ParamType::Address => EthType::Address,
            ParamType::Byte(_) => EthType::Bytes,
            _ => bail!("Unknown type: {}", value.tp.to_string()),
        })
    }
}
