#![allow(clippy::assign_op_pattern)]
#![allow(clippy::ptr_offset_with_cast)]

use crate::abi::inc_ret_param::types::ParamType;
use crate::abi::inc_ret_param::Param as AbiType;
use crate::abi::{Entry, FunHash};
use crate::bytecode::hir::stack::FRAME_SIZE;
use anyhow::{bail, Error};
use std::cmp::Ordering;
use std::ops::{Div, Rem};
use uint::construct_uint;

#[derive(Default, Debug)]
pub struct Env {
    fun: Function,
}

impl Env {
    pub fn new(fun: Function) -> Env {
        Env { fun }
    }

    pub fn call_data_size(&self) -> U256 {
        U256::from(self.fun.input.len() * FRAME_SIZE + self.fun.hash.as_ref().len())
    }

    pub fn hash(&self) -> FunHash {
        self.fun.hash
    }
}

#[derive(Default, Clone, Debug)]
pub struct Function {
    pub hash: FunHash,
    pub name: String,
    pub input: Vec<EthType>,
    pub output: Vec<EthType>,
}

#[derive(Debug, Clone)]
pub enum EthType {
    U256,
    Bool,
}

impl<'a> TryFrom<&'a AbiType> for EthType {
    type Error = Error;

    fn try_from(value: &'a AbiType) -> Result<Self, Self::Error> {
        Ok(match value.tp {
            ParamType::Bool => EthType::Bool,
            ParamType::UInt(_) | ParamType::Int(_) => EthType::U256,
            _ => bail!("Unknown type: {}", value.tp.to_string()),
        })
    }
}

impl<'a> TryFrom<(FunHash, &'a Entry)> for Function {
    type Error = Error;

    fn try_from((hash, entry): (FunHash, &'a Entry)) -> Result<Self, Self::Error> {
        Ok(Function {
            hash,
            name: entry.name().unwrap_or_default(),
            input: entry.inputs().map_or(Ok(Vec::new()), |inp| {
                inp.iter()
                    .map(EthType::try_from)
                    .collect::<Result<Vec<_>, _>>()
            })?,
            output: entry.outputs().map_or(Ok(Vec::new()), |out| {
                out.iter()
                    .map(EthType::try_from)
                    .collect::<Result<Vec<_>, _>>()
            })?,
        })
    }
}

construct_uint! {
    pub struct U256(4);
}

construct_uint! {
    pub struct U512(8);
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct I256(pub bool, pub U256);

impl I256 {
    pub fn min_value() -> I256 {
        I256(true, (U256::MAX & SIGN) + U256::from(1))
    }
}

impl Default for I256 {
    fn default() -> I256 {
        I256(false, U256::zero())
    }
}

impl Ord for I256 {
    fn cmp(&self, other: &I256) -> Ordering {
        match (self.0, other.0) {
            (false, false) => self.1.cmp(&other.1),
            (true, true) => self.1.cmp(&other.1).reverse(),
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
        }
    }
}

const SIGN: U256 = U256([
    0xffffffffffffffff,
    0xffffffffffffffff,
    0xffffffffffffffff,
    0x7fffffffffffffff,
]);

impl PartialOrd for I256 {
    fn partial_cmp(&self, other: &I256) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<U256> for I256 {
    fn from(val: U256) -> I256 {
        if val == U256::zero() {
            I256::default()
        } else if val & SIGN == val {
            I256(false, val)
        } else {
            I256(true, !val + U256::from(1u64))
        }
    }
}

impl From<I256> for U256 {
    fn from(value: I256) -> U256 {
        if value.0 {
            !value.1 + U256::from(1u64)
        } else {
            value.1
        }
    }
}

impl Div for I256 {
    type Output = I256;

    fn div(self, other: I256) -> I256 {
        if other == I256::default() {
            return I256::default();
        }

        let min_value = I256::min_value();
        if self == min_value && other.1 == U256::from(1) {
            return min_value;
        }

        let d = (self.1 / other.1) & SIGN;
        if d == U256::zero() {
            return I256::default();
        }
        I256(self.0 ^ other.0, d)
    }
}

impl Rem for I256 {
    type Output = I256;

    fn rem(self, other: I256) -> I256 {
        let r = (self.1 % other.1) & SIGN;

        if r == U256::zero() {
            return I256::default();
        }

        I256(self.0, r)
    }
}

impl From<U256> for U512 {
    fn from(value: U256) -> U512 {
        let arr = value.0;
        let mut ret = [0; 8];
        ret[0] = arr[0];
        ret[1] = arr[1];
        ret[2] = arr[2];
        ret[3] = arr[3];
        U512(ret)
    }
}

impl From<U512> for U256 {
    fn from(value: U512) -> U256 {
        let arr = value.0;
        if arr[4] | arr[5] | arr[6] | arr[7] != 0 {
            panic!("U512 is too big to fit in U256");
        }
        let mut ret = [0; 4];
        ret[0] = arr[0];
        ret[1] = arr[1];
        ret[2] = arr[2];
        ret[3] = arr[3];
        U256(ret)
    }
}
