use anyhow::{bail, ensure, Result};

use crate::abi::inc_ret_param::types::ParamType;
use crate::abi::inc_ret_param::value::ParamValue;

pub enum ValueEncode {
    // int265[3][2], int265[3], byte3, int, uint, bool
    Static(Vec<u8>),
    // int265[3][], int265[], bytes
    Dinamic(Vec<u8>),
}

impl ValueEncode {
    pub fn data_ref(&self) -> &Vec<u8> {
        match self {
            ValueEncode::Dinamic(data) | ValueEncode::Static(data) => data,
        }
    }

    pub fn len(&self) -> usize {
        self.data_ref().len()
    }
}

pub fn encode_value(
    value: &ParamValue,
    value_type: Option<&ParamType>,
    start: u32,
) -> Result<ValueEncode> {
    match value {
        ParamValue::Bool(value) => {
            let mut result = [0u8; 32];
            result[31] = if *value { 1 } else { 0 };
            Ok(ValueEncode::Static(result.to_vec()))
        }
        ParamValue::Int { value, .. } => {
            let value_bytes = value.to_be_bytes();
            Ok(ValueEncode::Static(pad_left32(&value_bytes).to_vec()))
        }
        ParamValue::UInt { value, .. } => {
            let value_bytes = value.to_be_bytes();
            Ok(ValueEncode::Static(pad_left32(&value_bytes).to_vec()))
        }
        ParamValue::String(..) => todo!(),
        ParamValue::Byte(data) => Ok(ValueEncode::Static(pad_right32(data).to_vec())),
        ParamValue::Bytes(data) => {
            // len
            let mut result = pad_left32(&data.len().to_be_bytes()).to_vec();
            // + value
            result.extend(pad_right32(&data));
            Ok(ValueEncode::Dinamic(result))
        }
        ParamValue::Address(..) => todo!(),
        ParamValue::Array(data) => {
            todo!("@todo ParamValue::Array")
            // if let Some(tp) = value_type {
            //     let (size, subtp) = match tp {
            //         ParamType::Array { size, tp: subtp } => (size, subtp),
            //         _ => bail!("Expected array. Type passed: {tp:?}"),
            //     };
            //     let venc = vec![Some(subtp.as_ref()); data.len()]
            //         .into_iter()
            //         .zip(&*data)
            //         .collect();
            //     if let Some(size) = size {
            //         ensure!(
            //             data.len() == *size as usize,
            //             "Invalid array length. Expected {tp:?}"
            //         );
            //         vec_encode(venc, start)
            //     } else {
            //         todo!("sub array");
            //         // size + value
            //         // let mut result = pad_left32(&data.len().to_be_bytes()).to_vec();
            //         // result.append(&mut value);
            //         // Ok(result)
            //     }
            // } else {
            //     let venc = vec![None; data.len()].into_iter().zip(&*data).collect();
            //     vec_encode(venc, start)
            // }
        }
        ParamValue::Custom { .. } => todo!(),
    }
}

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

fn vec_encode(data: Vec<(Option<&ParamType>, &ParamValue)>, mut start: u32) -> Result<Vec<u8>> {
    todo!()
    // let result = data
    //     .iter()
    //     .map(|(tp, item)| {
    //         let r = encode_value(item, *tp, start)?;
    //         start += r.len() as u32;
    //         Ok(r)
    //     })
    //     .collect::<Result<Vec<_>>>()?
    //     .into_iter()
    //     .flatten()
    //     .collect();
    // Ok(result)
}

trait ParamTypeSize {
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
            ParamType::Custom(name) => todo!(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::abi::call::encode::{encode_value, ParamTypeSize};
    use crate::abi::inc_ret_param::types::ParamType;
    use crate::abi::inc_ret_param::value::{AsParamValue, ParamValue};

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

    /// https://docs.soliditylang.org/en/v0.8.0/abi-spec.html#examples
    #[test]
    fn test_encode() {
        // true
        assert_eq!(
            hex::encode(encode_value(&true.to_param(), None, 0).unwrap().data_ref()),
            "0000000000000000000000000000000000000000000000000000000000000001",
        );

        // false
        assert_eq!(
            hex::encode(encode_value(&false.to_param(), None, 0).unwrap().data_ref()),
            "0000000000000000000000000000000000000000000000000000000000000000",
        );

        assert_eq!(
            hex::encode(encode_value(&69u32.to_param(), None, 0).unwrap().data_ref()),
            "0000000000000000000000000000000000000000000000000000000000000045",
        );

        // bytes3["abc","def"])
        assert_eq!(
            hex::encode(encode_value(&ParamValue::Array(vec![
                ParamValue::Byte("abc".as_bytes().to_vec()),
                ParamValue::Byte("def".as_bytes().to_vec()),
            ]),None,0).unwrap().data_ref()),
            "61626300000000000000000000000000000000000000000000000000000000006465660000000000000000000000000000000000000000000000000000000000"
        );

        // bytes("dove")
        // len + value
        assert_eq!(
            hex::encode(encode_value(&ParamValue::Bytes("dave".as_bytes().to_vec()),None,0).unwrap().data_ref()),
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
            hex::encode(encode_value(&value,Some(&tp),0).unwrap().data_ref()),
            "000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003"
        );

        // Dynamic size
        let tp = ParamType::Array {
            size: None,
            tp: Box::new(ParamType::UInt(256)),
        };
        assert_eq!(
            hex::encode(encode_value(&value,Some(&tp),0).unwrap().data_ref()),
            "0000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000003"
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
            hex::encode(encode_value(&value,Some(&tp),0).unwrap().data_ref()),
            "000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000004560000000000000000000000000000000000000000000000000000000000000789"
        );
        let tp = ParamType::Array {
            size: Some(2),
            tp: Box::new(ParamType::UInt(32)),
        };
        assert_eq!(
            hex::encode(encode_value(&value,Some(&tp),0).unwrap().data_ref()),
            "00000000000000000000000000000000000000000000000000000000000004560000000000000000000000000000000000000000000000000000000000000789"
        );
        assert_eq!(
            hex::encode(encode_value(&value,None,0).unwrap().data_ref()),
            "00000000000000000000000000000000000000000000000000000000000004560000000000000000000000000000000000000000000000000000000000000789"
        );

        // bytes
        let tp = ParamType::Bytes;
        let value = ParamValue::Bytes("Hello, world!".as_bytes().to_vec());
        assert_eq!(
            hex::encode(encode_value(&value,Some(&tp),0).unwrap().data_ref()),
            "000000000000000000000000000000000000000000000000000000000000000d48656c6c6f2c20776f726c642100000000000000000000000000000000000000"
        );

        // g(uint[][],string[])

        // uint[][]
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
            hex::encode(encode_value(&value, Some(&tp), 64).unwrap().data_ref()),
            "0000000000000000000000000000000000000000000000000000000000000040\
            00000000000000000000000000000000000000000000000000000000000000a0\
            0000000000000000000000000000000000000000000000000000000000000002\
            0000000000000000000000000000000000000000000000000000000000000001\
            0000000000000000000000000000000000000000000000000000000000000002\
            0000000000000000000000000000000000000000000000000000000000000001\
            0000000000000000000000000000000000000000000000000000000000000003"
        );
    }
}
