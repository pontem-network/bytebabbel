use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

use anyhow::{ensure, Result};
use ethabi::token::{LenientTokenizer, Tokenizer};
use ethabi::{Bytes, Constructor, Function, Token};
use primitive_types::U256;

// =================================================================================================

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

pub fn to_token(data: &(ethabi::ParamType, &str)) -> Result<Token, ethabi::Error> {
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

pub fn to_eth_address(data: &[u8]) -> [u8; 20] {
    let mut result = [0u8; 20];
    result[20 - data.len()..20].copy_from_slice(data);
    result
}

// =================================================================================================
pub const FUN_HASH_LEN: usize = 4;

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Default)]
pub struct FunHash([u8; FUN_HASH_LEN]);

impl FunHash {
    pub fn as_frame(&self) -> U256 {
        let mut buf = [0u8; 32];
        buf[0..4].copy_from_slice(&self.0);
        U256::from(&buf)
    }
}

impl AsRef<[u8; FUN_HASH_LEN]> for FunHash {
    fn as_ref(&self) -> &[u8; FUN_HASH_LEN] {
        &self.0
    }
}

impl From<[u8; FUN_HASH_LEN]> for FunHash {
    fn from(hash: [u8; FUN_HASH_LEN]) -> Self {
        FunHash(hash)
    }
}

impl Debug for FunHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl Display for FunHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod test {
    use ethabi::Contract;

    use crate::abi::call::EthEncodeByString;

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
