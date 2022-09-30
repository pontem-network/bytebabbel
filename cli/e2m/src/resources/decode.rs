use std::str::FromStr;
use std::string::ToString;

use anyhow::{anyhow, bail, ensure, Error, Result};
use clap::ArgEnum;
use serde_json::Value;

use move_core_types::account_address::AccountAddress;

#[derive(ArgEnum, Clone, Copy, Debug)]
pub(crate) enum U256Decode {
    String,
    Address,
}

impl ToString for U256Decode {
    fn to_string(&self) -> String {
        match self {
            U256Decode::String => "string",
            U256Decode::Address => "address",
        }
        .to_string()
    }
}

impl FromStr for U256Decode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "string" => Ok(U256Decode::String),
            "address" => Ok(U256Decode::Address),
            _ => bail!("Invalid params. {s}"),
        }
    }
}

pub(crate) fn replace_u256_to_numstring(json: &mut Value) -> Result<()> {
    match json {
        Value::Array(val) => val
            .iter_mut()
            .map(replace_u256_to_numstring)
            .collect::<Result<_>>()?,
        Value::Object(val) => {
            if is_object_u256(val) {
                *json = Value::String(object_to_u256(val)?.to_string());
            } else {
                val.iter_mut()
                    .map(|(.., val)| replace_u256_to_numstring(val))
                    .collect::<Result<_>>()?
            }
        }
        _ => (),
    };
    Ok(())
}

pub(crate) fn replace_u256_to_address(json: &mut Value) -> Result<()> {
    match json {
        Value::Array(val) => val
            .iter_mut()
            .map(replace_u256_to_address)
            .collect::<Result<_>>()?,
        Value::Object(val) => {
            if is_object_u256(val) {
                *json = Value::String(object_to_u256_address(val)?.to_hex_literal());
            } else {
                val.iter_mut()
                    .map(|(.., val)| replace_u256_to_address(val))
                    .collect::<Result<_>>()?
            }
        }
        _ => (),
    };
    Ok(())
}

fn is_object_u256(val: &serde_json::Map<String, Value>) -> bool {
    let keys: Vec<&String> = val.keys().collect();
    keys == vec!["v0", "v1", "v2", "v3"]
}

fn object_to_u256(val: &serde_json::Map<String, Value>) -> Result<primitive_types::U256> {
    let list_u64 = val
        .iter()
        .filter_map(|(.., value)| value.as_str())
        .map(|val: &str| val.parse::<u64>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| anyhow!("{err}.\nParsed to u256: {val:?}"))?;
    ensure!(list_u64.len() == 4, "Parsed to u256: {val:?}");

    let mut k = [0; 4];
    k.copy_from_slice(&list_u64);
    Ok(primitive_types::U256(k))
}

fn object_to_u256_address(val: &serde_json::Map<String, Value>) -> Result<AccountAddress> {
    let mut list_u64 = val
        .iter()
        .filter_map(|(.., value)| value.as_str())
        .map(|val: &str| val.parse::<u64>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| anyhow!("{err}.\nParsed to u256: {val:?}"))?;
    ensure!(list_u64.len() == 4, "Parsed to u256: {val:?}");
    list_u64.reverse();

    let addres_bytes: Vec<u8> = list_u64
        .into_iter()
        .map(|v| v.to_be_bytes())
        .flatten()
        .collect();
    Ok(AccountAddress::from_bytes(&addres_bytes)?)
}

#[cfg(test)]
mod test {
    use serde_json::Value;

    use crate::resources::decode::{replace_u256_to_address, replace_u256_to_numstring};

    const JSON_U256_TO_NUMSTRING: &str = r#"[
        {
          "v0": "1",
          "v1": "0",
          "v2": "0",
          "v3": "0"
        },
        {
          "v0": "4477988020393345024",
          "v1": "542101086",
          "v2": "0",
          "v3": "0"
        }
    ]"#;

    const JSON_U256_TO_ADDRESS: &str = r#"[
        {
          "v0": "8097560366627688448",
          "v1": "17357102489901502592",
          "v2": "857870592",
          "v3": "0"
        },
        {
          "v0": "10410825456312197172",
          "v1": "2093886094006521369",
          "v2": "3245222349",
          "v3": "0"
        },
        {
          "v0": "11178216322179337398",
          "v1": "14567165539185629335",
          "v2": "3291684990",
          "v3": "0"
        }
    ]"#;

    #[test]
    fn test_replace_u256() {
        let mut resp = serde_json::from_str(JSON_U256_TO_NUMSTRING).unwrap();
        replace_u256_to_numstring(&mut resp).unwrap();

        assert_eq!(
            resp,
            serde_json::from_str::<Value>(r#"[ "1", "10000000000000000000000000000"]"#).unwrap()
        );
    }

    #[test]
    fn test_replace_u256_address() {
        let mut resp = serde_json::from_str(JSON_U256_TO_ADDRESS).unwrap();
        replace_u256_to_address(&mut resp).unwrap();

        assert_eq!(
            resp,
            serde_json::from_str::<Value>(
                r#"[ 
                "0x33221100f0e0d0c0b0a090807060504030201000", 
                "0xc16e29cd1d0efa4f01638e19907aae97f6152034",
                "0xc433207eca28f7e0b37898979b21008531f90cb6"
            ]"#
            )
            .unwrap()
        );
    }
}
