use anyhow::{bail, Result};

use crate::abi::inc_ret_param::value::ParamValue;

pub trait ParamValueToRustType {
    fn to_bool(&self) -> Result<bool>;
    fn to_vec(&self) -> Result<Vec<ParamValue>>;
    fn to_isize(&self) -> Result<isize>;
    fn to_usize(&self) -> Result<usize>;
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

    fn to_isize(&self) -> Result<isize> {
        match self {
            ParamValue::Int { value, .. } => Ok(*value),
            ParamValue::UInt { value, .. } => Ok(*value as isize),
            _ => bail!("Expected UInt or Int type. Passed {self:?}"),
        }
    }

    fn to_usize(&self) -> Result<usize> {
        match self {
            ParamValue::Int { value, .. } => Ok(*value as usize),
            ParamValue::UInt { value, .. } => Ok(*value),
            _ => bail!("Expected UInt or Int type. Passed {self:?}"),
        }
    }
}
