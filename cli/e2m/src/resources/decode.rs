use std::fmt::Debug;

use anyhow::{anyhow, ensure, Result};
use eth::abi::call::fn_params_str_split;
use ethabi::ParamType;
use serde_json::Value;

// = = =

pub(crate) fn decode(mut json: Value, mask: &[String]) -> Result<Value> {
    mask.iter()
        .filter_map(|v| {
            let (index, types) = v.split_once(':')?;
            let types = fn_params_str_split(types)
                .map_err(err)
                .ok()?
                .into_iter()
                .map(ethabi::param_type::Reader::read)
                .collect::<std::result::Result<Vec<_>, _>>()
                .map_err(err)
                .ok()?;

            Some((index, types))
        })
        .try_for_each(|(index, mask)| decode_proc(&mut json, index, &mask))?;
    Ok(json)
}

fn decode_proc(json: &mut Value, index: &str, mask: &[ParamType]) -> Result<()> {
    if index == "*" {
        return replace_by_mask(json, mask);
    }

    match json {
        Value::Array(val) => val
            .iter_mut()
            .try_for_each(|val| decode_proc(val, index, mask))?,
        Value::Object(val) => val.iter_mut().try_for_each(|(val_index, val)| {
            if val_index == index {
                replace_by_mask(val, mask)
            } else {
                decode_proc(val, index, mask)
            }
        })?,
        _ => (),
    };
    Ok(())
}

fn replace_by_mask(json: &mut Value, mask: &[ParamType]) -> Result<()> {
    if let Err(err) = object_to_u256(json) {
        log::error!("{err:?}");
    }

    match json {
        Value::String(val) => {
            if val.starts_with("0x") {
                log::trace!("val: {val}");

                let result = hex::decode(val.trim_start_matches("0x"))
                    .ok()
                    .and_then(|bytes| ethabi::decode(mask, &bytes).map_err(err).ok());

                if let Some(result) = result {
                    log::trace!("decode: {result:?}");
                    *json = Value::String(format!("{result:?}"));
                }
            }
        }
        Value::Array(val) => val
            .iter_mut()
            .try_for_each(|val| replace_by_mask(val, mask))?,
        Value::Object(val) => val
            .iter_mut()
            .try_for_each(|(.., val)| replace_by_mask(val, mask))?,
        _ => (),
    }

    Ok(())
}

fn object_to_u256(json: &mut Value) -> Result<()> {
    if !json.is_object() {
        return Ok(());
    }

    let val = match json.as_object() {
        Some(val) => val,
        None => return Ok(()),
    };
    let keys: Vec<&String> = val.keys().collect();
    if keys != vec!["v0", "v1", "v2", "v3"] {
        return Ok(());
    }

    let mut list_u64 = val
        .iter()
        .filter_map(|(.., value)| value.as_str())
        .map(|val: &str| val.parse::<u64>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| anyhow!("{err}.\nParsed to u256: {val:?}"))?;
    ensure!(list_u64.len() == 4, "Parsed to u256: {val:?}");
    list_u64.reverse();

    let u256_bytes: Vec<u8> = list_u64.into_iter().flat_map(|v| v.to_be_bytes()).collect();

    let hex = format!("0x{}", hex::encode(&u256_bytes));
    *json = Value::String(hex);

    Ok(())
}

fn err<T>(err: T)
where
    T: Debug,
{
    log::trace!("Err: {err:?}")
}

// = = =

#[cfg(test)]
mod test {
    use serde_json::Value;

    use crate::resources::decode::decode;

    const JSON_TEST: &str = r#"[
          {
            "version": "420684",
            "guid": {
              "creation_number": "4",
              "account_address": "0xb9efc292b81e426405ebf74ac16e29cd1d0efa4f01638e19907aae97f6152034"
            },
            "sequence_number": "0",
            "type": "0xb9efc292b81e426405ebf74ac16e29cd1d0efa4f01638e19907aae97f6152034::Users::Event",
            "data": {
              "data": "0x000000000000000000000000c433207eca28f7e0b37898979b21008531f90cb600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
              "topics": [
                {
                  "v0": "4853473898109565021",
                  "v1": "12423157531212965247",
                  "v2": "16082917701485615544",
                  "v3": "14887007877718918608"
                }
              ]
            }
          },
          {
            "version": "421362",
            "guid": {
              "creation_number": "4",
              "account_address": "0xb9efc292b81e426405ebf74ac16e29cd1d0efa4f01638e19907aae97f6152034"
            },
            "sequence_number": "1",
            "type": "0xb9efc292b81e426405ebf74ac16e29cd1d0efa4f01638e19907aae97f6152034::Users::Event",
            "data": {
              "data": "0x000000000000000000000000c16e29cd1d0efa4f01638e19907aae97f6152034000000000000000000000000c433207eca28f7e0b37898979b21008531f90cb600000000000000000000000000000000000000000000000000000000000000c8",
              "topics": [
                {
                  "v0": "2951364421682967535",
                  "v1": "10748869590852608278",
                  "v2": "7620847484418887082",
                  "v3": "15992936130196719771"
                }
              ]
            }
          },
          {
            "version": "421626",
            "guid": {
              "creation_number": "4",
              "account_address": "0xb9efc292b81e426405ebf74ac16e29cd1d0efa4f01638e19907aae97f6152034"
            },
            "sequence_number": "2",
            "type": "0xb9efc292b81e426405ebf74ac16e29cd1d0efa4f01638e19907aae97f6152034::Users::Event",
            "data": {
              "data": "0x000000000000000000000000c16e29cd1d0efa4f01638e19907aae97f6152034000000000000000000000000c433207eca28f7e0b37898979b21008531f90cb600000000000000000000000000000000000000000000000000000000000000c8",
              "topics": [
                {
                  "v0": "2951364421682967535",
                  "v1": "10748869590852608278",
                  "v2": "7620847484418887082",
                  "v3": "15992936130196719771"
                }
              ]
            }
          }
        ]
    "#;

    #[test]
    fn test_decode() {
        let resp: Value = serde_json::from_str(JSON_TEST).unwrap();
        let result = decode(resp, &["data:address,address,u256".to_string()]).unwrap();

        let json_string = serde_json::to_string_pretty(&result).unwrap();
        assert!(json_string.contains("[Address(0xc433207eca28f7e0b37898979b21008531f90cb6), Address(0x0000000000000000000000000000000000000000), Uint(0)]"));
        assert!(json_string.contains("[Address(0xc16e29cd1d0efa4f01638e19907aae97f6152034), Address(0xc433207eca28f7e0b37898979b21008531f90cb6), Uint(200)]"));
    }
}
