use crate::abi::Param as AbiType;
use crate::abi::{Entry, FunHash};
use crate::bytecode::executor::stack::FRAME_SIZE;
use crate::bytecode::executor::types::U256;
use anyhow::{bail, Error};

#[derive(Default)]
pub struct Env {
    fun: Function,
}

impl Env {
    pub fn new(fun: Function) -> Env {
        Env { fun }
    }

    pub fn call_data_size(&self) -> U256 {
        U256::from(self.fun.input_size.len() * FRAME_SIZE + self.fun.hash.as_ref().len())
    }

    pub fn hash(&self) -> FunHash {
        self.fun.hash
    }
}

#[derive(Default)]
pub struct Function {
    pub hash: FunHash,
    pub name: String,
    pub input_size: Vec<EthType>,
    pub output_size: Vec<EthType>,
}

pub enum EthType {
    U256,
    Bool,
}

impl<'a> TryFrom<&'a AbiType> for EthType {
    type Error = Error;

    fn try_from(value: &'a AbiType) -> Result<Self, Self::Error> {
        Ok(match value.tp.as_str() {
            "bool" => EthType::Bool,
            "uint" | "uint256" => EthType::U256,
            _ => bail!("Unknown type: {}", value.tp),
        })
    }
}

impl<'a> TryFrom<(FunHash, &'a Entry)> for Function {
    type Error = Error;

    fn try_from((hash, entry): (FunHash, &'a Entry)) -> Result<Self, Self::Error> {
        Ok(Function {
            hash,
            name: entry.name(),
            input_size: entry
                .inputs()
                .iter()
                .map(EthType::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            output_size: entry
                .outputs()
                .unwrap()
                .iter()
                .map(EthType::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}
