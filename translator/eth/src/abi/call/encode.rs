use anyhow::{anyhow, bail, ensure, Result};
use evm_core::utils::I256;
use primitive_types::U256;

use ethabi::token::{LenientTokenizer, Tokenizer};
use ethabi::{Bytes, Constructor, Function, Token};

use crate::abi::inc_ret_param::types::ParamType;
use crate::abi::inc_ret_param::value::type_to_value::fn_params_str_split;
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
            let value = U256::from(*value);
            let mut value_bytes = vec![0u8; 32];
            value.to_big_endian(&mut value_bytes);
            Ok(ValueEncodeType::Static(value_bytes))
        }
        ParamValue::UInt { value, .. } => {
            let mut value_bytes = vec![0u8; 32];
            value.to_big_endian(&mut value_bytes);
            Ok(ValueEncodeType::Static(value_bytes))
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
            value: to_i256(value),
        }),
        ParamType::UInt(size) => Ok(ParamValue::UInt {
            size: *size,
            value: to_u256(value),
        }),
        ParamType::Byte(size) => {
            let data = { &value[0..*size as usize] }.to_vec();
            ensure!(*size as usize == data.len(), "incorrect length");
            Ok(ParamValue::Byte(data))
        }
        ParamType::Bytes | ParamType::String => {
            let len = to_u256(&value[0..32]).as_usize();
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
                    let size = to_u256(&value[0..32]);
                    value = &value[32..];
                    size.as_usize()
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
                    let offset = to_u256(&value[32 * index..32 * (index + 1)]).as_usize();
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

pub fn to_u256(data: &[u8]) -> U256 {
    U256::from_big_endian(data)
}

fn to_i256(data: &[u8]) -> I256 {
    U256::from_big_endian(data).into()
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

// =================================================================================================

trait EthEncodeConstructor {
    fn short_signature(&self) -> [u8; 4];
}

impl EthEncodeConstructor for Constructor {
    fn short_signature(&self) -> [u8; 4] {
        let params: Vec<_> = self.inputs.iter().map(|param| param.kind.clone()).collect();
        ethabi::short_signature("anonymous", &params)
    }
}

pub trait EthEncodeByString {
    fn short_signature_in_hex(&self) -> String;
    fn call_by_vec_str(&self, params: &[&str]) -> Result<Bytes>;
    fn call_by_str(&self, params: &str) -> Result<Bytes> {
        let params = fn_params_str_split(params)?;
        self.call_by_vec_str(&params)
    }
}

impl EthEncodeByString for Function {
    fn call_by_vec_str(&self, params: &[&str]) -> Result<Bytes> {
        let params = self
            .inputs
            .iter()
            .map(|param| param.kind.clone())
            .zip(params.iter().map(|v| v as &str))
            .collect::<Vec<_>>();
        let tokens: Vec<Token> = params.iter().map(to_token).collect::<Result<_, _>>()?;
        let result = self.encode_input(&tokens)?;
        Ok(result)
    }

    fn short_signature_in_hex(&self) -> String {
        hex::encode(self.short_signature())
    }
}

impl EthEncodeByString for Constructor {
    fn call_by_vec_str(&self, params: &[&str]) -> Result<Bytes> {
        let params = self
            .inputs
            .iter()
            .map(|param| param.kind.clone())
            .zip(params.iter().map(|v| v as &str))
            .collect::<Vec<_>>();
        let tokens: Vec<Token> = params.iter().map(to_token).collect::<Result<_, _>>()?;

        let result = self.encode_input(self.short_signature().into(), &tokens)?;
        Ok(result)
    }

    fn short_signature_in_hex(&self) -> String {
        hex::encode(self.short_signature())
    }
}

fn to_token(data: &(ethabi::ParamType, &str)) -> Result<Token, ethabi::Error> {
    match data.0 {
        ethabi::ParamType::Address => {
            let value = data.1.trim_start_matches("0x");

            let value = if value.len() >= 40 {
                value[value.len() - 40..].to_string()
            } else {
                "0".repeat(40 - value.len()) + value
            };
            LenientTokenizer::tokenize(&data.0, &value)
        }
        _ => LenientTokenizer::tokenize(&data.0, data.1),
    }
}

#[cfg(test)]
mod test {
    use crate::abi::call::encode::EthEncodeByString;
    use ethabi::Contract;

    /// Encoding and decoding input/output
    ///
    /// https://docs.soliditylang.org/en/v0.8.0/abi-spec.html#examples
    /// ============================================================================================
    /// // SPDX-License-Identifier: GPL-3.0
    /// pragma solidity >=0.4.16 <0.9.0;
    ///
    /// contract Foo {
    ///    function bar(bytes3[2] memory) public pure {}
    ///    function baz(uint32 x, bool y) public pure returns (bool r) { r = x > 32 || y; }
    ///    function sam(bytes memory, bool, uint[] memory) public pure {}
    /// }
    /// ============================================================================================
    #[test]
    fn test_input_encode() {
        let abi_str = r#"[
          {
            "inputs": [
              {
                "internalType": "bytes3[2]",
                "name": "",
                "type": "bytes3[2]"
              }
            ],
            "name": "bar",
            "outputs": [],
            "stateMutability": "pure",
            "type": "function"
          },
          {
            "inputs": [
              {
                "internalType": "uint32",
                "name": "x",
                "type": "uint32"
              },
              {
                "internalType": "bool",
                "name": "y",
                "type": "bool"
              }
            ],
            "name": "baz",
            "outputs": [
              {
                "internalType": "bool",
                "name": "r",
                "type": "bool"
              }
            ],
            "stateMutability": "pure",
            "type": "function"
          },
          {
            "inputs": [
              {
                "internalType": "bytes",
                "name": "",
                "type": "bytes"
              },
              {
                "internalType": "bool",
                "name": "",
                "type": "bool"
              },
              {
                "internalType": "uint256[]",
                "name": "",
                "type": "uint256[]"
              }
            ],
            "name": "sam",
            "outputs": [],
            "stateMutability": "pure",
            "type": "function"
          },
          {
            "inputs": [
              {
                "internalType": "uint256",
                "name": "",
                "type": "uint256"
              },
              {
                "internalType": "uint32[]",
                "name": "",
                "type": "uint32[]"
              },
              {
                "internalType": "bytes10",
                "name": "",
                "type": "bytes10"
              },
              {
                "internalType": "bytes",
                "name": "",
                "type": "bytes"
              }
            ],
            "name": "f",
            "outputs": [],
            "stateMutability": "pure",
            "type": "function"
          },
          {
            "inputs": [
              {
                "internalType": "uint256[][]",
                "name": "",
                "type": "uint256[][]"
              },
              {
                "internalType": "string[]",
                "name": "",
                "type": "string[]"
              }
            ],
            "name": "g",
            "outputs": [],
            "stateMutability": "pure",
            "type": "function"
          }
        ]"#;

        let abi: Contract = serde_json::from_str(abi_str).unwrap();

        // =========================================================================================
        // function baz(uint32 x, bool y)
        // =========================================================================================
        let entry_fn = abi.functions_by_name("baz").unwrap().first().unwrap();
        let encode = hex::encode(entry_fn.call_by_vec_str(&["69", "true"]).unwrap());
        assert_eq!(&entry_fn.short_signature_in_hex(), "cdcd77c0");
        assert_eq!(
            "0xcdcd77c0\
            0000000000000000000000000000000000000000000000000000000000000045\
            0000000000000000000000000000000000000000000000000000000000000001",
            format!("0x{encode}")
        );

        // =========================================================================================
        // function bar(bytes3[2] memory)
        // =========================================================================================
        let entry_fn = abi.functions_by_name("bar").unwrap().first().unwrap();
        let encode = hex::encode(
            entry_fn
                .call_by_vec_str(&[&format!(
                    "[0x{},0x{}]",
                    hex::encode("abc".as_bytes()),
                    hex::encode("def".as_bytes())
                )])
                .unwrap(),
        );

        assert_eq!(&entry_fn.short_signature_in_hex(), "fce353f6");
        assert_eq!(
            "0xfce353f6\
            6162630000000000000000000000000000000000000000000000000000000000\
            6465660000000000000000000000000000000000000000000000000000000000",
            format!("0x{encode}")
        );

        // =========================================================================================
        // function sam(bytes memory, bool, uint[] memory)
        // sam("dave",true,[1,2,3])
        // =========================================================================================
        let entry_fn = abi.functions_by_name("sam").unwrap().first().unwrap();
        let encode = hex::encode(
            entry_fn
                .call_by_vec_str(&[&hex::encode("dave".as_bytes()), "true", "[1,2,3]"])
                .unwrap(),
        );

        assert_eq!(&entry_fn.short_signature_in_hex(), "a5643bf2");
        assert_eq!(
            format!("0x{encode}"),
            "0xa5643bf2\
            0000000000000000000000000000000000000000000000000000000000000060\
            0000000000000000000000000000000000000000000000000000000000000001\
            00000000000000000000000000000000000000000000000000000000000000a0\
            0000000000000000000000000000000000000000000000000000000000000004\
            6461766500000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000003\
            0000000000000000000000000000000000000000000000000000000000000001\
            0000000000000000000000000000000000000000000000000000000000000002\
            0000000000000000000000000000000000000000000000000000000000000003",
        );

        // =========================================================================================
        // f(uint,uint32[],bytes10,bytes)
        // f(0x123, [0x456, 0x789], "1234567890", "Hello, world!")
        // =========================================================================================
        let entry_fn = abi.functions_by_name("f").unwrap().first().unwrap();
        let encode = hex::encode(
            entry_fn
                .call_by_vec_str(&[
                    "291",
                    "[1110,1929]",
                    &hex::encode("1234567890".as_bytes()),
                    &hex::encode("Hello, world!".as_bytes()),
                ])
                .unwrap(),
        );

        assert_eq!(&entry_fn.short_signature_in_hex(), "8be65246");
        assert_eq!(
            format!("0x{encode}"),
            "0x8be65246\
            0000000000000000000000000000000000000000000000000000000000000123\
            0000000000000000000000000000000000000000000000000000000000000080\
            3132333435363738393000000000000000000000000000000000000000000000\
            00000000000000000000000000000000000000000000000000000000000000e0\
            0000000000000000000000000000000000000000000000000000000000000002\
            0000000000000000000000000000000000000000000000000000000000000456\
            0000000000000000000000000000000000000000000000000000000000000789\
            000000000000000000000000000000000000000000000000000000000000000d\
            48656c6c6f2c20776f726c642100000000000000000000000000000000000000",
        );

        // =========================================================================================
        // g(uint[][],string[])
        // g([[1, 2], [3]], ["one", "two", "three"])
        // =========================================================================================
        let entry_fn = abi.functions_by_name("g").unwrap().first().unwrap();
        let encode = hex::encode(
            entry_fn
                .call_by_vec_str(&["[[1,2],[3]]", r#"[one,two,three]"#])
                .unwrap(),
        );

        assert_eq!(&entry_fn.short_signature_in_hex(), "2289b18c");
        assert_eq!(
            format!("0x{encode}"),
            "0x2289b18c\
            0000000000000000000000000000000000000000000000000000000000000040\
            0000000000000000000000000000000000000000000000000000000000000140\
            0000000000000000000000000000000000000000000000000000000000000002\
            0000000000000000000000000000000000000000000000000000000000000040\
            00000000000000000000000000000000000000000000000000000000000000a0\
            0000000000000000000000000000000000000000000000000000000000000002\
            0000000000000000000000000000000000000000000000000000000000000001\
            0000000000000000000000000000000000000000000000000000000000000002\
            0000000000000000000000000000000000000000000000000000000000000001\
            0000000000000000000000000000000000000000000000000000000000000003\
            0000000000000000000000000000000000000000000000000000000000000003\
            0000000000000000000000000000000000000000000000000000000000000060\
            00000000000000000000000000000000000000000000000000000000000000a0\
            00000000000000000000000000000000000000000000000000000000000000e0\
            0000000000000000000000000000000000000000000000000000000000000003\
            6f6e650000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000003\
            74776f0000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000005\
            7468726565000000000000000000000000000000000000000000000000000000",
        );
    }
}
