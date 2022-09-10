use crate::abi::inc_ret_param::types::ParamType;
use anyhow::{bail, Result};
use evm_core::utils::I256;
use itertools::Itertools;
use primitive_types::U256;
use std::fmt::Debug;

// bytes,byte1,byte2,byte<N>.., address and array
pub mod collection;
pub mod conv;
pub mod number;
pub mod type_to_value;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ParamValue {
    Bool(bool),
    // 2^3...2^8 = 8 ... 256
    // default: 256
    Int {
        size: u16,
        value: I256,
    },
    // 2^3...2^8 = 8 ... 256
    // default: 256
    UInt {
        size: u16,
        value: U256,
    },
    // 1 ... 32
    Byte(Vec<u8>),
    // Holds a 20 byte value (size of an Ethereum address).
    Address([u8; 32]),
    // Dynamically-sized byte array
    Bytes(Vec<u8>),
    // Dynamically-sized byte array
    String(Vec<u8>),
    Array(Vec<ParamValue>),
    // Not a Primitive type
    Custom {
        name: String,
        params: Vec<ParamValue>,
    },
}

impl ParamValue {
    /// Int and UInt
    pub fn size(&self) -> Result<usize> {
        match self {
            ParamValue::Int { size, .. } | ParamValue::UInt { size, .. } => Ok(*size as usize),
            _ => bail!("It is not possible to get a value for this type"),
        }
    }

    /// Int and UInt
    pub fn set_size(&mut self, new_size: usize) -> Result<()> {
        match self {
            ParamValue::Int { size, .. } | ParamValue::UInt { size, .. } => {
                *size = new_size as u16;
                Ok(())
            }
            _ => bail!("You cannot change the size of this type"),
        }
    }

    /// Bytes, Array
    pub fn length(&self) -> Result<usize> {
        match self {
            ParamValue::Bytes(data) => Ok(data.len()),
            ParamValue::Array(data) => Ok(data.len()),
            _ => bail!("It is not possible to get a value for this type"),
        }
    }

    pub fn to_type(&self) -> Result<ParamType> {
        match self {
            ParamValue::Bool(..) => Ok(ParamType::Bool),
            ParamValue::Int { size, .. } => Ok(ParamType::Int(*size)),
            ParamValue::UInt { size, .. } => Ok(ParamType::UInt(*size)),
            ParamValue::String(..) => Ok(ParamType::String),
            ParamValue::Byte(data) => Ok(ParamType::Byte(data.len() as u8)),
            ParamValue::Bytes(..) => Ok(ParamType::Bytes),
            ParamValue::Address(..) => Ok(ParamType::Address),
            ParamValue::Array(data) => {
                let mut value = data
                    .iter()
                    .map(|item| item.to_type())
                    .collect::<Result<Vec<ParamType>>>()?;

                let count_sizes = value
                    .iter()
                    .map(|item| match item {
                        ParamType::Array { size, .. } => size.unwrap_or_default(),
                        _ => 0,
                    })
                    .unique()
                    .count();

                if count_sizes != 1 {
                    bail!("Different sizes of collections")
                }

                let size = value.len();
                let tp = value.remove(0);

                let result = ParamType::Array {
                    tp: Box::new(tp),
                    size: Some(size as u32),
                };
                Ok(result)
            }
            ParamValue::Custom { .. } => todo!(),
        }
    }
}

pub trait AsParamValue: Debug {
    fn to_param(self) -> ParamValue;

    fn try_to_param_bool(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        bail!("Expected bool type. Not implemented")
    }

    fn try_to_param_int(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        bail!("Expected Int type. Not implemented")
    }

    fn try_to_param_string(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        bail!("Expected &str type. Not implemented");
    }

    fn try_to_param_array(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        bail!("Expected [..] type. Not implemented");
    }

    fn try_to_array_by_type(self, _tp: &ParamType) -> Result<ParamValue>
    where
        Self: Sized,
    {
        bail!("Expected [..] type. Not implemented");
    }

    fn try_to_param_uint(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        bail!("Expected UInt type. Not implemented")
    }

    fn try_to_vec_u8(self) -> Result<Vec<u8>>
    where
        Self: Sized,
    {
        bail!("Expected vec[u8] or &[u8] type. Not implemented");
    }
}

impl<T: AsParamValue> From<T> for ParamValue {
    fn from(v: T) -> Self {
        v.to_param()
    }
}

impl AsParamValue for &str {
    fn to_param(self) -> ParamValue {
        ParamValue::String(self.as_bytes().to_vec())
    }

    fn try_to_param_string(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        Ok(self.to_param())
    }
}

impl AsParamValue for bool {
    fn to_param(self) -> ParamValue {
        ParamValue::Bool(self)
    }

    fn try_to_param_bool(self) -> Result<ParamValue>
    where
        Self: Sized,
    {
        Ok(self.to_param())
    }
}

#[cfg(test)]
mod test {
    use crate::abi::inc_ret_param::value::{AsParamValue, ParamValue};

    #[test]
    fn test_to_param_bool() {
        assert_eq!(ParamValue::from(true), ParamValue::Bool(true));
        assert_eq!(true.to_param(), ParamValue::Bool(true));
        assert_eq!(ParamValue::from(false), ParamValue::Bool(false));
        assert_eq!(false.to_param(), ParamValue::Bool(false));
    }

    #[test]
    fn test_to_param_string() {
        let string = "demo";
        assert_eq!(
            ParamValue::from(string),
            ParamValue::String(string.as_bytes().to_vec())
        );
        assert_eq!(
            string.to_param(),
            ParamValue::String(string.as_bytes().to_vec())
        );
    }
}
