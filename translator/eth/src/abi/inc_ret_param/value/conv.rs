use anyhow::{bail, Result};
use evm_core::utils::I256;
use primitive_types::U256;

use crate::abi::inc_ret_param::value::ParamValue;

pub trait ParamValueToRustType {
    fn to_bool(&self) -> Result<bool>;
    fn to_vec(&self) -> Result<Vec<ParamValue>>;
    fn to_i256(&self) -> Result<I256>;
    fn to_u256(&self) -> Result<U256>;
}

impl ParamValueToRustType for ParamValue {
    fn to_bool(&self) -> Result<bool> {
        if let ParamValue::Bool(value) = self {
            Ok(*value)
        } else {
            bail!("Expected Bool type. Passed {self:?}");
        }
    }

    fn to_vec(&self) -> Result<Vec<ParamValue>> {
        if let ParamValue::Array(data) = self {
            Ok(data.clone())
        } else {
            bail!("Expected Array type. Passed {self:?}");
        }
    }

    fn to_i256(&self) -> Result<I256> {
        match self {
            ParamValue::Int { value, .. } => Ok(*value),
            ParamValue::UInt { value, .. } => Ok(I256::from(*value)),
            _ => bail!("Expected UInt or Int type. Passed {self:?}"),
        }
    }

    fn to_u256(&self) -> Result<U256> {
        match self {
            ParamValue::Int { value, .. } => Ok(U256::from(*value)),
            ParamValue::UInt { value, .. } => Ok(*value),
            _ => bail!("Expected UInt or Int type. Passed {self:?}"),
        }
    }
}
