use anyhow::{bail, Result};

use crate::abi::inc_ret_param::types::ParamType;
use crate::abi::inc_ret_param::value::collection::{TryParamAddress, TryParamBytes};
use crate::abi::inc_ret_param::value::{AsParamValue, ParamValue};

impl ParamType {
    pub fn set_value<T>(&self, value: T) -> Result<ParamValue>
    where
        T: AsParamValue,
    {
        match self {
            ParamType::Bool => value.try_to_param_bool(),
            ParamType::Int(size) => {
                let size = *size;

                let mut result = value.try_to_param_int()?;
                let rsize = result.size().unwrap_or_default() as u16;

                if size == rsize {
                    return Ok(result);
                }
                if size >= 128 {
                    result.set_size(size as usize)?;
                    return Ok(result);
                }
                bail!("Type i{size} was expected")
            }
            ParamType::UInt(size) => {
                let size = *size;
                let mut result = value.try_to_param_uint()?;
                let rsize = result.size().unwrap_or_default() as u16;

                if size == rsize {
                    return Ok(result);
                }
                if size >= 128 {
                    result.set_size(size as usize)?;
                    return Ok(result);
                }
                bail!("Type u{size} was expected")
            }
            ParamType::Array { .. } => value.try_to_array_by_type(self),
            ParamType::String => value.try_to_param_string(),
            ParamType::Bytes => {
                let v = value.try_to_vec_u8()?;
                Ok(v.try_to_param_bytes())
            }
            ParamType::Byte(size) => {
                let v = value.try_to_vec_u8()?;
                v.try_to_param_bytes_with_size(*size as usize)
            }
            ParamType::Address => {
                let v = value.try_to_vec_u8()?;
                v.try_as_param_address()
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::abi::inc_ret_param::types::ParamType;
    use crate::abi::inc_ret_param::value::ParamValue;

    #[test]
    fn type_set_value_bool() {
        let tp = ParamType::Bool;

        assert_eq!(tp.set_value(true).unwrap(), ParamValue::Bool(true));
        assert!(tp.set_value(1).is_err());
    }

    #[test]
    fn type_set_value_int() {
        let tp = ParamType::Int(8);

        assert_eq!(
            tp.set_value(1i8).unwrap(),
            ParamValue::Int { size: 8, value: 1 }
        );
        assert!(tp.set_value(1isize).is_err());
        assert!(tp.set_value(true).is_err());

        let tp = ParamType::Int(128);
        assert_eq!(
            tp.set_value(1i8).unwrap(),
            ParamValue::Int {
                size: 128,
                value: 1
            }
        );
        assert_eq!(
            tp.set_value(2i128).unwrap(),
            ParamValue::Int {
                size: 128,
                value: 2
            }
        );
        assert_eq!(
            tp.set_value(3isize).unwrap(),
            ParamValue::Int {
                size: 128,
                value: 3
            }
        );

        let tp = ParamType::Int(256);
        assert_eq!(
            tp.set_value(1i8).unwrap(),
            ParamValue::Int {
                size: 256,
                value: 1
            }
        );
        assert_eq!(
            tp.set_value(2i128).unwrap(),
            ParamValue::Int {
                size: 256,
                value: 2
            }
        );
        assert_eq!(
            tp.set_value(3isize).unwrap(),
            ParamValue::Int {
                size: 256,
                value: 3
            }
        );
    }

    #[test]
    fn type_set_value_uint() {
        let tp = ParamType::UInt(8);

        assert_eq!(
            tp.set_value(1u8).unwrap(),
            ParamValue::UInt { size: 8, value: 1 }
        );
        assert!(tp.set_value(1isize).is_err());
        assert!(tp.set_value(true).is_err());

        let tp = ParamType::UInt(128);
        assert_eq!(
            tp.set_value(1u8).unwrap(),
            ParamValue::UInt {
                size: 128,
                value: 1
            }
        );
        assert_eq!(
            tp.set_value(2u128).unwrap(),
            ParamValue::UInt {
                size: 128,
                value: 2
            }
        );
        assert_eq!(
            tp.set_value(3usize).unwrap(),
            ParamValue::UInt {
                size: 128,
                value: 3
            }
        );

        let tp = ParamType::UInt(256);
        assert_eq!(
            tp.set_value(1u8).unwrap(),
            ParamValue::UInt {
                size: 256,
                value: 1
            }
        );
        assert_eq!(
            tp.set_value(2u128).unwrap(),
            ParamValue::UInt {
                size: 256,
                value: 2
            }
        );
        assert_eq!(
            tp.set_value(3usize).unwrap(),
            ParamValue::UInt {
                size: 256,
                value: 3
            }
        );
    }

    #[test]
    fn type_set_value_str() {
        let tp = ParamType::String;
        assert_eq!(
            tp.set_value("demo").unwrap(),
            ParamValue::String("demo".as_bytes().to_vec())
        );

        assert!(tp.set_value([1, 2, 3]).is_err());
    }

    #[test]
    fn type_set_value_array() {
        let tp = ParamType::Array {
            size: Some(2),
            tp: Box::new(ParamType::UInt(8)),
        };
        let result = ParamValue::Array(vec![
            ParamValue::UInt { size: 8, value: 1 },
            ParamValue::UInt { size: 8, value: 2 },
        ]);
        assert_eq!(tp.set_value([1u8, 2]).unwrap(), result);
        assert_eq!(tp.set_value(vec![1u8, 2]).unwrap(), result);
        assert_eq!(tp.set_value([1u8, 2].as_slice()).unwrap(), result);
        assert!(tp.set_value([1, 2]).is_err());
        assert!(tp.set_value([1]).is_err());

        let tp = ParamType::Array {
            size: None,
            tp: Box::new(ParamType::UInt(256)),
        };
        let result = ParamValue::Array(vec![ParamValue::UInt {
            size: 256,
            value: 1,
        }]);
        assert_eq!(tp.set_value([1usize]).unwrap(), result);
        assert!(tp.set_value([1]).is_err());

        let tp = ParamType::Array {
            size: None,
            tp: Box::new(ParamType::Array {
                size: Some(2),
                tp: Box::new(ParamType::Int(8)),
            }),
        };

        let result = ParamValue::Array(vec![
            ParamValue::Array(vec![
                ParamValue::Int { size: 8, value: 1 },
                ParamValue::Int { size: 8, value: 2 },
            ]),
            ParamValue::Array(vec![
                ParamValue::Int { size: 8, value: 3 },
                ParamValue::Int { size: 8, value: 4 },
            ]),
        ]);
        assert_eq!(tp.set_value([[1i8, 2], [3, 4]]).unwrap(), result);
        assert!(tp.set_value([[1, 2], [3, 4]]).is_err());

        // string
        let tp = ParamType::Array {
            size: Some(2),
            tp: Box::new(ParamType::String),
        };
        let result = ParamValue::Array(vec![
            ParamValue::String("1".as_bytes().to_vec()),
            ParamValue::String("2".as_bytes().to_vec()),
        ]);
        assert_eq!(tp.set_value(["1", "2"]).unwrap(), result);
    }

    #[test]
    fn type_set_value_bytes() {
        let tp = ParamType::Bytes;
        assert_eq!(
            tp.set_value(vec![1, 2]).unwrap(),
            ParamValue::Bytes(vec![1, 2])
        );
        assert_eq!(tp.set_value([1, 2]).unwrap(), ParamValue::Bytes(vec![1, 2]));
        assert_eq!(
            tp.set_value([1, 2].as_slice()).unwrap(),
            ParamValue::Bytes(vec![1, 2])
        );

        let tp = ParamType::Byte(3);
        assert_eq!(
            tp.set_value([1, 2, 3]).unwrap(),
            ParamValue::Byte(vec![1, 2, 3])
        );
        assert!(tp.set_value([1, 2]).is_err());
    }

    #[test]
    fn type_set_value_address() {
        let tp = ParamType::Address;
        assert_eq!(tp.set_value([0; 32]).unwrap(), ParamValue::Address([0; 32]));

        assert!(tp.set_value([1, 2, 3]).is_err());
    }
}
