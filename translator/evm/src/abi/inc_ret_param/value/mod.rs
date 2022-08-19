use crate::abi::inc_ret_param::types::ParamType;
use anyhow::{bail, ensure, Result};
use itertools::Itertools;
use std::fmt::Debug;

// bytes,byte1,byte2,byte<N>.., address and array
pub mod collection;
pub mod number;
pub mod type_to_value;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ParamValue {
    Bool(bool),
    // 2^3...2^8 = 8 ... 256
    // default: 256
    Int {
        size: u16,
        value: isize,
    },
    // 2^3...2^8 = 8 ... 256
    // default: 256
    UInt {
        size: u16,
        value: usize,
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
    // @todo
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

    pub fn encode(&self, tp: Option<&ParamType>) -> Result<Vec<u8>> {
        match self {
            ParamValue::Bool(value) => {
                let mut result = [0u8; 32];
                result[31] = if *value { 1 } else { 0 };
                Ok(result.to_vec())
            }
            ParamValue::Int { value, .. } => {
                let value_bytes = value.to_be_bytes();
                Ok(pad_left32(&value_bytes).to_vec())
            }
            ParamValue::UInt { value, .. } => {
                let value_bytes = value.to_be_bytes();
                Ok(pad_left32(&value_bytes).to_vec())
            }
            ParamValue::String(..) => todo!(),
            ParamValue::Byte(data) => Ok(pad_right32(data).to_vec()),
            ParamValue::Bytes(data) => {
                // len
                let mut result = pad_left32(&data.len().to_be_bytes()).to_vec();
                // + value
                result.extend(pad_right32(&data));
                Ok(result)
            }
            ParamValue::Address(..) => todo!(),
            ParamValue::Array(data) => {
                if let Some(tp) = tp {
                    let (size, subtp) = match tp {
                        ParamType::Array { size, tp: subtp } => (size, subtp),
                        _ => bail!("Expected array. Type passed: {tp:?}"),
                    };
                    let venc = vec![Some(subtp.as_ref()); data.len()]
                        .into_iter()
                        .zip(&*data)
                        .collect();
                    let mut value = vec_encode(venc)?;
                    if let Some(size) = size {
                        ensure!(
                            data.len() == *size as usize,
                            "Invalid array length. Expected {tp:?}"
                        );
                        Ok(value)
                    } else {
                        // size + value
                        let mut result = pad_left32(&data.len().to_be_bytes()).to_vec();
                        result.append(&mut value);
                        Ok(result)
                    }
                } else {
                    let venc = vec![None; data.len()].into_iter().zip(&*data).collect();
                    vec_encode(venc)
                }
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

// =================================================================================================
fn pad_left32(data: &[u8]) -> [u8; 32] {
    let mut result = [0u8; 32];
    result[32 - data.len()..32].copy_from_slice(&data);
    result
}

fn pad_right32(data: &[u8]) -> [u8; 32] {
    let mut result = [0u8; 32];
    result[0..data.len()].copy_from_slice(&data);
    result
}

fn vec_encode(data: Vec<(Option<&ParamType>, &ParamValue)>) -> Result<Vec<u8>> {
    let result = data
        .iter()
        .map(|(tp, item)| item.encode(*tp))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect();
    Ok(result)
}

#[cfg(test)]
mod test {
    use crate::abi::inc_ret_param::types::ParamType;
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

    /// https://docs.soliditylang.org/en/v0.8.0/abi-spec.html#examples
    #[test]
    fn test_encode() {
        // true
        assert_eq!(
            hex::encode(true.to_param().encode(None).unwrap()),
            "0000000000000000000000000000000000000000000000000000000000000001",
        );

        // false
        assert_eq!(
            hex::encode(false.to_param().encode(None).unwrap()),
            "0000000000000000000000000000000000000000000000000000000000000000",
        );

        assert_eq!(
            hex::encode(69u32.to_param().encode(None).unwrap()),
            "0000000000000000000000000000000000000000000000000000000000000045",
        );

        // bytes3["abc","def"])
        assert_eq!(
            hex::encode(ParamValue::Array(vec![
                ParamValue::Byte("abc".as_bytes().to_vec()),
                ParamValue::Byte("def".as_bytes().to_vec()),
            ]).encode(None).unwrap()),
            "61626300000000000000000000000000000000000000000000000000000000006465660000000000000000000000000000000000000000000000000000000000"
        );

        // bytes("dove")
        // len + value
        assert_eq!(
            hex::encode(ParamValue::Bytes("dave".as_bytes().to_vec()).encode(None).unwrap()),
            "00000000000000000000000000000000000000000000000000000000000000046461766500000000000000000000000000000000000000000000000000000000",
        );

        // uint256[1,2,3]
        let value = ParamValue::Array(vec![
            ParamValue::UInt {
                size: 256,
                value: 1,
            },
            ParamValue::UInt {
                size: 256,
                value: 2,
            },
            ParamValue::UInt {
                size: 256,
                value: 3,
            },
        ]);

        // Fixed size
        let tp = ParamType::Array {
            size: Some(3),
            tp: Box::new(ParamType::UInt(256)),
        };

        assert_eq!(
            hex::encode(value.encode(Some(&tp)).unwrap()),
            "000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003"
        );

        // Dynamic size
        let tp = ParamType::Array {
            size: None,
            tp: Box::new(ParamType::UInt(256)),
        };
        assert_eq!(
            hex::encode(value.encode(Some(&tp)).unwrap()),
            "0000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003"
        );
    }
}
