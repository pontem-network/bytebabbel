use anyhow::{anyhow, bail, ensure, Result};

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

pub fn decode_value(value: &[u8], value_type: &ParamType) -> Result<ParamValue> {
    match value_type {
        ParamType::Bool => {
            let b = value.last().ok_or_else(|| anyhow!("Value not passed"))?;
            let result = b != &0;
            Ok(ParamValue::Bool(result))
        }
        ParamType::Int(size) => Ok(ParamValue::Int {
            size: *size,
            value: to_isize(value),
        }),
        ParamType::UInt(size) => Ok(ParamValue::UInt {
            size: *size,
            value: to_usize(value),
        }),
        ParamType::Byte(size) => {
            let data = { &value[0..*size as usize] }.to_vec();
            ensure!(*size as usize == data.len(), "incorrect length");
            Ok(ParamValue::Byte(data))
        }
        ParamType::Bytes | ParamType::String => {
            let len = to_usize(&value[0..32]);
            let data = { &value[32..32 + len] }.to_vec();
            let result = match value_type {
                ParamType::Bytes => ParamValue::Bytes(data),
                ParamType::String => ParamValue::String(data),
                _ => unreachable!(),
            };
            Ok(result)
        }
        ParamType::Address => {
            let mut address = [0u8; 32];
            address.copy_from_slice(&value[..32]);
            Ok(ParamValue::Address(address))
        }
        ParamType::Array {
            size: len,
            tp: sub_type,
        } => {
            let mut value = value;
            let len = match len {
                Some(size) => *size as usize,
                None => {
                    let size = to_usize(&value[0..32]);
                    value = &value[32..];
                    size
                }
            };

            // static
            if sub_type.is_static_size() {
                let sub_size = sub_type.size_bytes().unwrap_or(32) as usize;
                let data = value
                    .chunks(sub_size)
                    .take(len)
                    .map(|value| decode_value(value, sub_type))
                    .collect::<Result<Vec<ParamValue>>>()?;

                return Ok(ParamValue::Array(data));
            }

            let result = (0..len)
                .map(|index| {
                    let offset = to_usize(&value[32 * index..32 * (index + 1)]);
                    decode_value(&value[offset..], sub_type)
                })
                .collect::<Result<Vec<ParamValue>>>()?;

            Ok(ParamValue::Array(result))
        }
        ParamType::Custom { .. } => todo!(),
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

pub fn to_usize(data: &[u8]) -> usize {
    let mut bytes = [0u8; 8];
    bytes.clone_from_slice(&data[24..32]);
    usize::from_be_bytes(bytes)
}

fn to_isize(data: &[u8]) -> isize {
    let mut bytes = [0u8; 8];
    bytes.clone_from_slice(&data[24..32]);
    isize::from_be_bytes(bytes)
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
    use crate::abi::call::encode::{decode_value, encode_value, ParamTypeSize};
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
        let tp = ParamType::Bool;

        for (value, enc) in [
            (
                true.to_param(),
                hex::decode("0000000000000000000000000000000000000000000000000000000000000001")
                    .unwrap(),
            ),
            (
                false.to_param(),
                hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
                    .unwrap(),
            ),
        ] {
            assert_eq!(encode_value(&value, &tp, 0).unwrap().data_ref(), &enc);
            assert_eq!(decode_value(&enc, &tp).unwrap(), value);
        }
    }

    #[test]
    fn test_encode_num() {
        let tp = ParamType::UInt(32);
        let value = 69u32.to_param();
        let enc = hex::decode("0000000000000000000000000000000000000000000000000000000000000045")
            .unwrap();

        assert_eq!(encode_value(&value, &tp, 0).unwrap().data_ref(), &enc);
        assert_eq!(decode_value(&enc, &tp).unwrap(), value);

        let tp = ParamType::Int(128);
        let value = { -69i128 }.to_param();
        let enc = hex::decode("000000000000000000000000000000000000000000000000ffffffffffffffbb")
            .unwrap();
        assert_eq!(encode_value(&value, &tp, 0).unwrap().data_ref(), &enc);
        assert_eq!(decode_value(&enc, &tp).unwrap(), value);
    }

    #[test]
    fn test_encode_bytes() {
        // bytes3["abc","def"])
        let tp = ParamType::Array {
            size: Some(2),
            tp: Box::new(ParamType::Byte(3)),
        };
        let value = ParamValue::Array(vec![
            ParamValue::Byte("abc".as_bytes().to_vec()),
            ParamValue::Byte("def".as_bytes().to_vec()),
        ]);
        let enc = hex::decode("61626300000000000000000000000000000000000000000000000000000000006465660000000000000000000000000000000000000000000000000000000000")
            .unwrap();

        assert_eq!(encode_value(&value, &tp, 0).unwrap().data_ref(), &enc);
        assert_eq!(decode_value(&enc, &tp).unwrap(), value);

        // bytes("dove")
        // len + value
        let tp = ParamType::Bytes;
        let value = ParamValue::Bytes("dave".as_bytes().to_vec());
        let enc = hex::decode("00000000000000000000000000000000000000000000000000000000000000046461766500000000000000000000000000000000000000000000000000000000")
            .unwrap();
        assert_eq!(encode_value(&value, &tp, 0).unwrap().data_ref(), &enc);
        assert_eq!(decode_value(&enc, &tp).unwrap(), value);

        let tp = ParamType::Bytes;
        let value = ParamValue::Bytes("Hello, world!".as_bytes().to_vec());
        let enc = hex::decode("000000000000000000000000000000000000000000000000000000000000000d48656c6c6f2c20776f726c642100000000000000000000000000000000000000").unwrap();
        assert_eq!(encode_value(&value, &tp, 0).unwrap().data_ref(), &enc);
        assert_eq!(decode_value(&enc, &tp).unwrap(), value);
    }

    #[test]
    fn test_encode_array() {
        // uint256[1,2,3]
        // Fixed size
        let tp = ParamType::Array {
            size: Some(3),
            tp: Box::new(ParamType::UInt(256)),
        };
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
        let enc = hex::decode(
            "0000000000000000000000000000000000000000000000000000000000000001\
            0000000000000000000000000000000000000000000000000000000000000002\
            0000000000000000000000000000000000000000000000000000000000000003",
        )
        .unwrap();
        assert_eq!(encode_value(&value, &tp, 0).unwrap().data_ref(), &enc);
        assert_eq!(decode_value(&enc, &tp).unwrap(), value);

        // Dynamic size
        let tp = ParamType::Array {
            size: None,
            tp: Box::new(ParamType::UInt(256)),
        };
        let enc = hex::decode(
            "0000000000000000000000000000000000000000000000000000000000000003\
            0000000000000000000000000000000000000000000000000000000000000001\
            0000000000000000000000000000000000000000000000000000000000000002\
            0000000000000000000000000000000000000000000000000000000000000003",
        )
        .unwrap();
        assert_eq!(encode_value(&value, &tp, 0).unwrap().data_ref(), &enc);
        assert_eq!(decode_value(&enc, &tp).unwrap(), value);

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
        let enc = hex::decode(
            "0000000000000000000000000000000000000000000000000000000000000002\
            0000000000000000000000000000000000000000000000000000000000000456\
            0000000000000000000000000000000000000000000000000000000000000789",
        )
        .unwrap();
        assert_eq!(encode_value(&value, &tp, 0).unwrap().data_ref(), &enc);
        assert_eq!(decode_value(&enc, &tp).unwrap(), value);

        let tp = ParamType::Array {
            size: Some(2),
            tp: Box::new(ParamType::UInt(32)),
        };
        let enc = hex::decode("00000000000000000000000000000000000000000000000000000000000004560000000000000000000000000000000000000000000000000000000000000789").unwrap() ;
        assert_eq!(encode_value(&value, &tp, 0).unwrap().data_ref(), &enc);
        assert_eq!(decode_value(&enc, &tp).unwrap(), value);

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
        let enc = hex::decode(
            "0000000000000000000000000000000000000000000000000000000000000002\
            0000000000000000000000000000000000000000000000000000000000000040\
            00000000000000000000000000000000000000000000000000000000000000a0\
            0000000000000000000000000000000000000000000000000000000000000002\
            0000000000000000000000000000000000000000000000000000000000000001\
            0000000000000000000000000000000000000000000000000000000000000002\
            0000000000000000000000000000000000000000000000000000000000000001\
            0000000000000000000000000000000000000000000000000000000000000003",
        )
        .unwrap();
        assert_eq!(encode_value(&value, &tp, 0).unwrap().data_ref(), &enc);
        assert_eq!(decode_value(&enc, &tp).unwrap(), value);

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
        let enc = hex::decode(
            "0000000000000000000000000000000000000000000000000000000000000003\
            0000000000000000000000000000000000000000000000000000000000000060\
            00000000000000000000000000000000000000000000000000000000000000a0\
            00000000000000000000000000000000000000000000000000000000000000e0\
            0000000000000000000000000000000000000000000000000000000000000003\
            6f6e650000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000003\
            74776f0000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000005\
            7468726565000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        assert_eq!(encode_value(&value, &tp, 0).unwrap().data_ref(), &enc);
        assert_eq!(decode_value(&enc, &tp).unwrap(), value);
    }
}
