use std::fmt::Debug;

use anyhow::{anyhow, ensure, Result};
use ethabi::{Contract, Event, ParamType};
use serde_json::Value;

use eth::abi::call::fn_params_str_split;

// = = =

pub(crate) fn decode_by_types(mut json: Value, mask: &[String]) -> Result<Value> {
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

pub(crate) fn decode_by_abi(json: &mut Value, abi: &Contract) {
    for event in abi.events.iter().flat_map(|(.., events)| events) {
        let signature = format!("{:#x}", event.signature());
        // let types: Vec<ParamType> = event.inputs.iter().map(|v| v.kind.clone()).collect();

        decode_event(json, &signature, event);
    }
}

// = = =

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
    if let Err(err) = object_to_hex(json) {
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

// = = =

/// Searching for the `topics` index.
/// This field contains the `hash` of the `event` from abi.
/// Decoding adjacent fields by the types of this event
fn decode_event(json: &mut Value, signature: &str, event: &Event) {
    match json {
        Value::Array(data) => data
            .iter_mut()
            .for_each(|v| decode_event(v, signature, event)),
        Value::Object(data) => {
            let topics = data.get_mut("topics").and_then(topics_to_hash_string);
            if topics.as_deref() == Some(signature) {
                decode_event_data(data, event);
            }

            data.iter_mut()
                .for_each(|(.., v)| decode_event(v, signature, event));
        }
        _ => (),
    }
}

fn decode_event_data(json: &mut serde_json::Map<String, Value>, event: &Event) {
    let types: Vec<ParamType> = event.inputs.iter().map(|t| t.kind.clone()).collect();
    let names: Vec<String> = event.inputs.iter().map(|t| t.name.clone()).collect();

    for (name, val) in json {
        if name == "topics" {
            continue;
        }
        if let Value::String(data) = val {
            if !data.starts_with("0x") {
                continue;
            }
            let result = hex::decode(data.trim_start_matches("0x"))
                .ok()
                .and_then(|bytes| ethabi::decode(&types, &bytes).map_err(err).ok());

            if let Some(result) = result {
                let mut map = serde_json::Map::new();
                result.iter().zip(&names).for_each(|(val, name)| {
                    map.insert(name.clone(), Value::String(format!("{val:?}")));
                });

                *val = Value::Object(map);
            }
        }
    }
}

fn topics_to_hash_string(json: &mut Value) -> Option<String> {
    object_to_hex(json).ok()?;

    match json {
        Value::Array(data) => data.iter_mut().map(topics_to_hash_string).next()?,
        Value::String(data) => {
            if data.starts_with("0x") {
                Some(data.clone())
            } else {
                None
            }
        }
        _ => None,
    }
}

// = = =

/// u256 object to hex
fn object_to_hex(json: &mut Value) -> Result<()> {
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

    let hex = format!("0x{}", hex::encode(u256_bytes));

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

    use crate::resources::decode::{decode_by_abi, decode_by_types};

    const RESPONSE: &str = include_str!("../../resources/tests/response_events.json");

    const ABI: &str = include_str!("../../resources/tests/users.abi");

    #[test]
    fn test_decode_type() {
        let resp: Value = serde_json::from_str(RESPONSE).unwrap();
        let result = decode_by_types(resp, &["data:address,address,u256".to_string()]).unwrap();

        let json_string = serde_json::to_string_pretty(&result).unwrap();

        assert!(json_string.contains("[Address(0xc433207eca28f7e0b37898979b21008531f90cb6), Address(0x0000000000000000000000000000000000000000), Uint(0)]"));
        assert!(json_string.contains("[Address(0xc16e29cd1d0efa4f01638e19907aae97f6152034), Address(0xc433207eca28f7e0b37898979b21008531f90cb6), Uint(200)]"));
    }

    #[test]
    fn test_decode_abi() {
        let mut response: Value = serde_json::from_str(RESPONSE).unwrap();
        let abi: ethabi::Contract = serde_json::from_str(ABI).unwrap();

        decode_by_abi(&mut response, &abi);
        let json_string = serde_json::to_string_pretty(&response).unwrap();

        assert!(json_string.contains(
            r#"{
        "addr": "Address(0xc433207eca28f7e0b37898979b21008531f90cb6)",
        "is_admin": "Bool(false)",
        "amount": "Uint(0)"
      }"#
        ));
        assert!(json_string.contains(
            r#"{
      "data": {
        "from": "Address(0xc16e29cd1d0efa4f01638e19907aae97f6152034)",
        "to": "Address(0xc433207eca28f7e0b37898979b21008531f90cb6)",
        "amount": "Uint(200)"
      }"#
        ));
    }
}
