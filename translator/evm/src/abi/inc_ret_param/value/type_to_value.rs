use anyhow::{bail, ensure, Result};

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

    pub fn set_value_str(&self, value: &str) -> Result<ParamValue> {
        let value = value.trim();
        match self {
            ParamType::Bool => self.set_value(value.parse::<bool>()?),
            ParamType::Int(size) => {
                let value = value.split_once("i").map(|(v, _)| v).unwrap_or(value);
                Ok(ParamValue::Int {
                    size: *size,
                    value: value.parse::<isize>()?,
                })
            }
            ParamType::UInt(size) => {
                let value = value.split_once("u").map(|(v, _)| v).unwrap_or(value);
                Ok(ParamValue::UInt {
                    size: *size,
                    value: value.parse::<usize>()?,
                })
            }
            ParamType::Array { size, tp } => {
                let value = &value[1..value.len() - 1];
                let sub = fn_params_str_split(value)?
                    .into_iter()
                    .map(|val| tp.set_value_str(val))
                    .collect::<Result<Vec<ParamValue>>>()?;

                if let Some(s) = size {
                    ensure!(sub.len() == *s as usize, "Expected array {self:?}");
                }

                Ok(ParamValue::Array(sub))
            }
            ParamType::String => self.set_value(value.trim_matches('"')),
            ParamType::Bytes | ParamType::Byte(..) => {
                self.set_value(hex::decode(value.trim_start_matches("0x"))?)
            }
            ParamType::Address => {
                let mut val = value.trim_start_matches("0x").to_string();
                if val.len() < 64 {
                    val = "0".repeat(64 - val.len()) + &val;
                }
                let bt = hex::decode(&val)?;
                ensure!(
                    bt.len() <= 32,
                    "The address cannot exceed 32 bytes.{self:?} {value}"
                );
                self.set_value(bt)
            }
            _ => unreachable!(),
        }
    }
}

pub fn fn_params_str_split(params: &str) -> Result<Vec<&str>> {
    let params = params.trim();

    let mut lf = 0;
    let mut quote = false;
    let mut esc = false;
    let mut last_pos = 0;

    let mut result: Vec<&str> = params
        .chars()
        .enumerate()
        .filter_map(|(pos, ch)| match ch {
            '\\' => {
                esc = !esc;
                None
            }
            _ if esc => {
                esc = false;
                None
            }
            '"' => {
                quote = !quote;
                None
            }
            _ if quote => None,
            '[' => {
                lf += 1;
                None
            }
            ']' => {
                lf -= 1;
                None
            }
            ',' => {
                if lf != 0 {
                    None
                } else {
                    let arg = params[last_pos..pos].trim();
                    last_pos = pos + 1;
                    Some(arg)
                }
            }
            _ => None,
        })
        .collect();

    ensure!(lf == 0, "Error when splitting a params {params}");

    if params.len() != last_pos {
        result.push(params[last_pos..params.len()].trim());
    }

    Ok(result)
}

#[cfg(test)]
mod test {
    use crate::abi::inc_ret_param::types::ParamType;
    use crate::abi::inc_ret_param::value::type_to_value::fn_params_str_split;
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
    fn test_params_str_split() {
        assert!(fn_params_str_split("").unwrap().is_empty());
        assert_eq!(fn_params_str_split("1").unwrap(), vec!["1"]);
        assert_eq!(
            fn_params_str_split(r#"1,"demo",0x1"#).unwrap(),
            vec!["1", r#""demo""#, "0x1"]
        );
        assert_eq!(
            fn_params_str_split(r#" [[1,2],[3]], "demo[1,[2]" "#).unwrap(),
            vec!["[[1,2],[3]]", r#""demo[1,[2]""#]
        );
        assert_eq!(
            fn_params_str_split(r#"1,"demo\"123\"",0x1"#).unwrap(),
            vec!["1", r#""demo\"123\"""#, "0x1"]
        );
        assert!(fn_params_str_split(r#" [[1,2],[3], "demo[1,2]" "#).is_err());
    }

    #[test]
    fn type_set_value_str_bool() {
        let tp = ParamType::Bool;

        assert_eq!(tp.set_value_str("true").unwrap(), ParamValue::Bool(true));
        assert!(tp.set_value_str("1").is_err());
    }

    #[test]
    fn type_set_value_str_int() {
        let tp = ParamType::Int(8);

        assert_eq!(
            tp.set_value_str("1").unwrap(),
            ParamValue::Int { size: 8, value: 1 }
        );
        assert_eq!(
            tp.set_value_str("1i8").unwrap(),
            ParamValue::Int { size: 8, value: 1 }
        );
        assert!(tp.set_value_str("true").is_err());

        let tp = ParamType::Int(128);
        assert_eq!(
            tp.set_value_str("3isize").unwrap(),
            ParamValue::Int {
                size: 128,
                value: 3
            }
        );

        let tp = ParamType::Int(256);
        assert_eq!(
            tp.set_value_str("3isize").unwrap(),
            ParamValue::Int {
                size: 256,
                value: 3
            }
        );
    }

    #[test]
    fn type_set_value_str_uint() {
        let tp = ParamType::UInt(8);

        assert_eq!(
            tp.set_value_str("1").unwrap(),
            ParamValue::UInt { size: 8, value: 1 }
        );
        assert_eq!(
            tp.set_value_str("1u8").unwrap(),
            ParamValue::UInt { size: 8, value: 1 }
        );
        assert!(tp.set_value_str("true").is_err());

        let tp = ParamType::UInt(128);
        assert_eq!(
            tp.set_value_str("2u128").unwrap(),
            ParamValue::UInt {
                size: 128,
                value: 2
            }
        );

        let tp = ParamType::UInt(256);
        assert_eq!(
            tp.set_value_str("3usize").unwrap(),
            ParamValue::UInt {
                size: 256,
                value: 3
            }
        );
    }

    #[test]
    fn type_set_value_str_str() {
        let tp = ParamType::String;
        assert_eq!(
            tp.set_value_str("demo").unwrap(),
            ParamValue::String("demo".as_bytes().to_vec())
        );
        assert_eq!(
            tp.set_value_str(r#""demo""#).unwrap(),
            ParamValue::String("demo".as_bytes().to_vec())
        );
    }

    #[test]
    fn type_set_value_str_array() {
        let tp = ParamType::Array {
            size: Some(2),
            tp: Box::new(ParamType::UInt(8)),
        };
        let result = ParamValue::Array(vec![
            ParamValue::UInt { size: 8, value: 1 },
            ParamValue::UInt { size: 8, value: 2 },
        ]);
        assert_eq!(tp.set_value_str("[1u8, 2]").unwrap(), result);
        assert_eq!(tp.set_value_str("[1, 2]").unwrap(), result);

        let tp = ParamType::Array {
            size: None,
            tp: Box::new(ParamType::UInt(256)),
        };
        let result = ParamValue::Array(vec![ParamValue::UInt {
            size: 256,
            value: 1,
        }]);
        assert_eq!(tp.set_value_str("[1usize]").unwrap(), result);
        assert_eq!(tp.set_value_str("[1]").unwrap(), result);

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
        assert_eq!(tp.set_value_str("[[1i8, 2], [3, 4]]").unwrap(), result);
        assert_eq!(tp.set_value_str("[[1, 2], [3, 4]]").unwrap(), result);

        // string
        let tp = ParamType::Array {
            size: Some(2),
            tp: Box::new(ParamType::String),
        };
        let result = ParamValue::Array(vec![
            ParamValue::String("1".as_bytes().to_vec()),
            ParamValue::String("2".as_bytes().to_vec()),
        ]);
        assert_eq!(tp.set_value_str(r#"["1", "2"]"#).unwrap(), result);
    }

    #[test]
    fn type_set_value_str_bytes() {
        let tp = ParamType::Bytes;
        let bt = "123".as_bytes().to_vec();
        let bt_hex = format!("0x{}", hex::encode(&bt));
        assert_eq!(tp.set_value_str(&bt_hex).unwrap(), ParamValue::Bytes(bt));

        let tp = ParamType::Byte(3);
        let bt = vec![1, 2, 3];
        let bt_hex = format!("0x{}", hex::encode(&bt));
        assert_eq!(tp.set_value_str(&bt_hex).unwrap(), ParamValue::Byte(bt));
    }

    #[test]
    fn type_set_value_str_address() {
        let tp = ParamType::Address;
        let mut bt = [0; 32];
        bt[31] = 1;

        assert_eq!(
            tp.set_value_str("0x0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap(),
            ParamValue::Address(bt)
        );
        assert_eq!(tp.set_value_str("0x1").unwrap(), ParamValue::Address(bt));
        assert_eq!(
            tp.set_value_str("0x00001").unwrap(),
            ParamValue::Address(bt)
        );
    }
}
