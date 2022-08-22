use anyhow::{bail, Result};

use crate::abi::inc_ret_param::types::ParamType;
use crate::abi::inc_ret_param::value::ParamValue;

pub enum ValueEncodeType {
    // int265[3][2], int265[3], byte3, int, uint, bool
    Static(Vec<u8>),
    // int265[3][], int265[], bytes
    Dynamic(Vec<u8>),
}

impl ValueEncodeType {
    pub fn data_ref(&self) -> &Vec<u8> {
        match self {
            ValueEncodeType::Dynamic(data) | ValueEncodeType::Static(data) => data,
        }
    }

    pub fn len(&self) -> usize {
        self.data_ref().len()
    }

    pub fn is_empty(&self) -> bool {
        self.data_ref().is_empty()
    }

    pub fn as_vec(self) -> Vec<u8> {
        match self {
            ValueEncodeType::Dynamic(data) | ValueEncodeType::Static(data) => data,
        }
    }
}

pub fn encode_value(
    value: &ParamValue,
    value_type: &ParamType,
    mut start: usize,
) -> Result<ValueEncodeType> {
    match value {
        ParamValue::Bool(value) => {
            let mut result = [0u8; 32];
            result[31] = if *value { 1 } else { 0 };
            Ok(ValueEncodeType::Static(result.to_vec()))
        }
        ParamValue::Int { value, .. } => {
            let value_bytes = value.to_be_bytes();
            Ok(ValueEncodeType::Static(pad_left32(&value_bytes).to_vec()))
        }
        ParamValue::UInt { value, .. } => {
            let value_bytes = value.to_be_bytes();
            Ok(ValueEncodeType::Static(pad_left32(&value_bytes).to_vec()))
        }
        ParamValue::Byte(data) => Ok(ValueEncodeType::Static(pad_right32(data).to_vec())),
        ParamValue::Bytes(data) | ParamValue::String(data) => {
            // dynamic
            let mut result = Vec::new();
            result.extend(pad_left32(&data.len().to_be_bytes()));
            for ch in data.chunks(32) {
                result.extend(pad_right32(ch))
            }
            Ok(ValueEncodeType::Dynamic(result))
        }
        ParamValue::Address(data) => Ok(ValueEncodeType::Static(data.to_vec())),
        ParamValue::Array(data) => {
            let sub_type = match value_type {
                ParamType::Array { tp: subtp, .. } => subtp,
                _ => bail!("Expected array. Type passed: {value_type:?}"),
            };

            // static
            if value_type.is_static_size() {
                let result = data
                    .iter()
                    .map(|item| encode_value(item, sub_type, start))
                    .collect::<Result<Vec<_>>>()?
                    .into_iter()
                    .flat_map(|item| item.as_vec())
                    .collect::<Vec<u8>>();
                return Ok(ValueEncodeType::Static(result));
            }

            // dynamic
            let mut result = Vec::new();

            result.extend(pad_left32(&data.len().to_be_bytes()));
            start += 32 * data.len();

            let subvalue = data
                .iter()
                .map(|value| {
                    if !sub_type.is_static_size() {
                        result.extend(pad_left32(&start.to_be_bytes()));
                    }
                    let vec = encode_value(value, sub_type, start)?.as_vec();
                    start += vec.len();
                    Ok(vec)
                })
                .collect::<Result<Vec<Vec<u8>>>>()?
                .into_iter()
                .flatten();
            result.extend(subvalue);
            Ok(ValueEncodeType::Dynamic(result))
        }
        ParamValue::Custom { .. } => todo!(),
    }
}

fn pad_left32(data: &[u8]) -> [u8; 32] {
    let mut result = [0u8; 32];
    result[32 - data.len()..32].copy_from_slice(data);
    result
}

fn pad_right32(data: &[u8]) -> [u8; 32] {
    let mut result = [0u8; 32];
    result[0..data.len()].copy_from_slice(data);
    result
}

pub fn enc_offset(start: u32) -> [u8; 32] {
    pad_left32(&start.to_be_bytes())
}

pub trait ParamTypeSize {
    fn size_bytes(&self) -> Option<u32>;
}

impl ParamTypeSize for ParamType {
    fn size_bytes(&self) -> Option<u32> {
        if !self.is_static_size() {
            return None;
        }

        match self {
            ParamType::Bool => Some(32),
            ParamType::Int(..) => Some(32),
            ParamType::UInt(..) => Some(32),
            ParamType::Byte(..) => Some(32),
            ParamType::Bytes => None,
            ParamType::Address => Some(32),
            ParamType::String => None,
            ParamType::Array { tp, size } => size.and_then(|mult| {
                let ch = tp.size_bytes()?;
                Some(mult * ch)
            }),
            ParamType::Custom(..) => todo!(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::abi::call::encode::{encode_value, ParamTypeSize};
    use crate::abi::inc_ret_param::types::ParamType;
    use crate::abi::inc_ret_param::value::{AsParamValue, ParamValue};

    // https://docs.soliditylang.org/en/v0.8.0/abi-spec.html#examples

    #[test]
    fn test_size_bytes() {
        assert_eq!(Some(32), ParamType::Bool.size_bytes());
        assert_eq!(Some(32), ParamType::Int(8).size_bytes());
        assert_eq!(Some(32), ParamType::UInt(16).size_bytes());
        assert_eq!(Some(32), ParamType::Byte(3).size_bytes());
        assert_eq!(Some(32), ParamType::Address.size_bytes());
        assert_eq!(None, ParamType::Bytes.size_bytes());
        assert_eq!(None, ParamType::String.size_bytes());

        assert_eq!(
            None,
            ParamType::Array {
                size: None,
                tp: Box::new(ParamType::UInt(8))
            }
            .size_bytes()
        );

        assert_eq!(
            Some(32 * 3),
            ParamType::Array {
                size: Some(3),
                tp: Box::new(ParamType::UInt(8))
            }
            .size_bytes()
        );

        assert_eq!(
            Some(32 * 3 * 3),
            ParamType::Array {
                size: Some(3),
                tp: Box::new(ParamType::Array {
                    size: Some(3),
                    tp: Box::new(ParamType::Bool),
                }),
            }
            .size_bytes()
        );

        assert_eq!(
            None,
            ParamType::Array {
                size: Some(3),
                tp: Box::new(ParamType::Array {
                    size: None,
                    tp: Box::new(ParamType::Bool),
                }),
            }
            .size_bytes()
        );

        assert_eq!(
            None,
            ParamType::Array {
                size: None,
                tp: Box::new(ParamType::Array {
                    size: Some(3),
                    tp: Box::new(ParamType::Bool),
                }),
            }
            .size_bytes()
        );
    }

    #[test]
    fn test_encode_bool() {
        // true
        assert_eq!(
            hex::encode(
                encode_value(&true.to_param(), &ParamType::Bool, 0)
                    .unwrap()
                    .data_ref()
            ),
            "0000000000000000000000000000000000000000000000000000000000000001",
        );

        // false
        assert_eq!(
            hex::encode(
                encode_value(&false.to_param(), &ParamType::Bool, 0)
                    .unwrap()
                    .data_ref()
            ),
            "0000000000000000000000000000000000000000000000000000000000000000",
        );
    }

    #[test]
    fn test_encode_num() {
        assert_eq!(
            hex::encode(
                encode_value(&69u32.to_param(), &ParamType::UInt(32), 0)
                    .unwrap()
                    .data_ref()
            ),
            "0000000000000000000000000000000000000000000000000000000000000045",
        );
    }

    #[test]
    fn test_encode_bytes() {
        // bytes3["abc","def"])
        assert_eq!(
            hex::encode(encode_value(&ParamValue::Array(vec![
                ParamValue::Byte("abc".as_bytes().to_vec()),
                ParamValue::Byte("def".as_bytes().to_vec()),
            ]),&ParamType::Array {
                size:Some(2),
                tp:Box::new(ParamType::Byte(3))
            },0).unwrap().data_ref()),
            "61626300000000000000000000000000000000000000000000000000000000006465660000000000000000000000000000000000000000000000000000000000"
        );

        // bytes("dove")
        // len + value
        assert_eq!(
            hex::encode(encode_value(&ParamValue::Bytes("dave".as_bytes().to_vec()),&ParamType::Bytes,0).unwrap().data_ref()),
            "00000000000000000000000000000000000000000000000000000000000000046461766500000000000000000000000000000000000000000000000000000000",
        );

        let tp = ParamType::Bytes;
        let value = ParamValue::Bytes("Hello, world!".as_bytes().to_vec());
        assert_eq!(
            hex::encode(encode_value(&value,&tp,0).unwrap().data_ref()),
            "000000000000000000000000000000000000000000000000000000000000000d48656c6c6f2c20776f726c642100000000000000000000000000000000000000"
        );
    }

    #[test]
    fn test_encode_array() {
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
            hex::encode(encode_value(&value, &tp, 0).unwrap().data_ref()),
            "0000000000000000000000000000000000000000000000000000000000000001\
            0000000000000000000000000000000000000000000000000000000000000002\
            0000000000000000000000000000000000000000000000000000000000000003"
        );

        // Dynamic size
        let tp = ParamType::Array {
            size: None,
            tp: Box::new(ParamType::UInt(256)),
        };
        assert_eq!(
            hex::encode(encode_value(&value, &tp, 0).unwrap().data_ref()),
            "0000000000000000000000000000000000000000000000000000000000000003\
            0000000000000000000000000000000000000000000000000000000000000001\
            0000000000000000000000000000000000000000000000000000000000000002\
            0000000000000000000000000000000000000000000000000000000000000003"
        );

        // uint32[]
        // uint32[1110, 1929]
        let tp = ParamType::Array {
            size: None,
            tp: Box::new(ParamType::UInt(32)),
        };
        let value = ParamValue::Array(vec![
            ParamValue::UInt {
                size: 32,
                value: 1110,
            },
            ParamValue::UInt {
                size: 32,
                value: 1929,
            },
        ]);
        assert_eq!(
            hex::encode(encode_value(&value, &tp, 0).unwrap().data_ref()),
            "0000000000000000000000000000000000000000000000000000000000000002\
            0000000000000000000000000000000000000000000000000000000000000456\
            0000000000000000000000000000000000000000000000000000000000000789"
        );
        let tp = ParamType::Array {
            size: Some(2),
            tp: Box::new(ParamType::UInt(32)),
        };
        assert_eq!(
            hex::encode(encode_value(&value,&tp,0).unwrap().data_ref()),
            "00000000000000000000000000000000000000000000000000000000000004560000000000000000000000000000000000000000000000000000000000000789"
        );

        // g(uint[][],string[])

        // uint[][]
        // [[1,2],[3]]
        let tp = ParamType::Array {
            size: None,
            tp: Box::new(ParamType::Array {
                size: None,
                tp: Box::new(ParamType::UInt(256)),
            }),
        };
        let value = ParamValue::Array(vec![
            ParamValue::Array(vec![
                ParamValue::UInt {
                    size: 256,
                    value: 1,
                },
                ParamValue::UInt {
                    size: 256,
                    value: 2,
                },
            ]),
            ParamValue::Array(vec![ParamValue::UInt {
                size: 256,
                value: 3,
            }]),
        ]);

        // 0 - a                                                                - offset of [1, 2]
        // 1 - b                                                                - offset of [3]
        // 2 - 0000000000000000000000000000000000000000000000000000000000000002 - count for [1, 2]
        // 3 - 0000000000000000000000000000000000000000000000000000000000000001 - encoding of 1
        // 4 - 0000000000000000000000000000000000000000000000000000000000000002 - encoding of 2
        // 5 - 0000000000000000000000000000000000000000000000000000000000000001 - count for [3]
        // 6 - 0000000000000000000000000000000000000000000000000000000000000003 - encoding of 3
        assert_eq!(
            hex::encode(encode_value(&value, &tp, 0).unwrap().data_ref()),
            "0000000000000000000000000000000000000000000000000000000000000002\
            0000000000000000000000000000000000000000000000000000000000000040\
            00000000000000000000000000000000000000000000000000000000000000a0\
            0000000000000000000000000000000000000000000000000000000000000002\
            0000000000000000000000000000000000000000000000000000000000000001\
            0000000000000000000000000000000000000000000000000000000000000002\
            0000000000000000000000000000000000000000000000000000000000000001\
            0000000000000000000000000000000000000000000000000000000000000003"
        );

        // string[]
        // ["one", "two", "three"]

        let tp = ParamType::Array {
            size: None,
            tp: Box::new(ParamType::String),
        };
        let value = ParamValue::Array(vec![
            ParamValue::String("one".as_bytes().to_vec()),
            ParamValue::String("two".as_bytes().to_vec()),
            ParamValue::String("three".as_bytes().to_vec()),
        ]);

        // 10 - 0000000000000000000000000000000000000000000000000000000000000003 - count for ["one", "two", "three"]
        // 11 - 0000000000000000000000000000000000000000000000000000000000000060 - offset for "one"
        // 12 - 00000000000000000000000000000000000000000000000000000000000000a0 - offset for "two"
        // 13 - 00000000000000000000000000000000000000000000000000000000000000e0 - offset for "three"
        // 14 - 0000000000000000000000000000000000000000000000000000000000000003 - count for "one"
        // 15 - 6f6e650000000000000000000000000000000000000000000000000000000000 - encoding of "one"
        // 16 - 0000000000000000000000000000000000000000000000000000000000000003 - count for "two"
        // 17 - 74776f0000000000000000000000000000000000000000000000000000000000 - encoding of "two"
        // 18 - 0000000000000000000000000000000000000000000000000000000000000005 - count for "three"
        // 19 - 7468726565000000000000000000000000000000000000000000000000000000 - encoding of "three"
        assert_eq!(
            hex::encode(encode_value(&value, &tp, 0).unwrap().data_ref()),
            "0000000000000000000000000000000000000000000000000000000000000003\
            0000000000000000000000000000000000000000000000000000000000000060\
            00000000000000000000000000000000000000000000000000000000000000a0\
            00000000000000000000000000000000000000000000000000000000000000e0\
            0000000000000000000000000000000000000000000000000000000000000003\
            6f6e650000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000003\
            74776f0000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000005\
            7468726565000000000000000000000000000000000000000000000000000000"
        );
    }
}
