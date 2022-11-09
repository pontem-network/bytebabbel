use std::collections::BTreeMap;
use std::{collections::HashMap, str::FromStr};

use anyhow::{anyhow, Result};
use reqwest::{header, Url};
use serde_json::{json, Value};

use aptos_types::{access_path::AccessPath, state_store::state_key::StateKey};
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, ResourceKey, StructTag},
};

mod response;

use crate::{
    load::response::{MoveModuleId, MoveResource},
    profile::ProfileConfig,
    resolver::{print_access_path::AccessPathToString, HandleRequest},
    MoveExecutor,
};

pub trait LoadRemoteData {
    fn load_all(&mut self, profile: &ProfileConfig, address: &AccountAddress) -> Result<()> {
        self.load_modules(profile, address)
            .and(self.load_resources(profile, address))
    }

    fn load_modules(&mut self, profile: &ProfileConfig, address: &AccountAddress) -> Result<()>;

    fn load_resources(&mut self, profile: &ProfileConfig, address: &AccountAddress) -> Result<()>;
}

impl LoadRemoteData for MoveExecutor {
    fn load_modules(&mut self, profile: &ProfileConfig, address: &AccountAddress) -> Result<()> {
        for (name, bytecode) in load_modules(profile, address)? {
            log::info!("loaded module: {}", name.print_string());
            self.resolver.state_data.insert(name, bytecode);
        }
        Ok(())
    }

    fn load_resources(&mut self, profile: &ProfileConfig, address: &AccountAddress) -> Result<()> {
        self.resolver.handler_data = load_handel_link(profile, address)?;

        for (name, bytecode) in load_resource(profile, address)? {
            log::info!("loaded resource: {}", name.print_string());
            self.resolver.state_data.insert(name, bytecode);
        }

        Ok(())
    }
}

/// https://fullnode.devnet.aptoslabs.com/v1/tables/{table_handle}/item
pub fn load_table_handle_u256(data: &HandleRequest, key: &Vec<u8>) -> Result<Option<Vec<u8>>> {
    log::info!("{data:?}");
    log::info!("{key:?}");

    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Content-Type",
        header::HeaderValue::from_static("application/json"),
    );
    headers.insert(
        "accept",
        header::HeaderValue::from_static("application/x-bcs"),
    );

    let u256_key = primitive_types::U256::from_little_endian(key.as_slice());
    let body = json!({
        "key_type":data.key_type,
        "value_type":data.value_type,
        "key":{
            "v0":format!("{}", &u256_key.0[0]),
            "v1":format!("{}", &u256_key.0[1]),
            "v2":format!("{}", &u256_key.0[2]),
            "v3":format!("{}", &u256_key.0[3]),
        }
    });
    if let Ok(b) = serde_json::to_string_pretty(&body) {
        log::trace!("{b}");
    }

    let result = match reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()?
        .post(data.url.as_ref())
        .body(body.to_string())
        .send()?
        .bytes()
    {
        Ok(bytes) => {
            let result = bytes.to_vec();
            match String::from_utf8(result.clone())
                .ok()
                .and_then(|msg| serde_json::from_str::<serde_json::Value>(&msg).ok())
            {
                Some(err) => {
                    log::error!("{}", serde_json::to_string_pretty(&err).unwrap());
                    None
                }
                None => Some(result),
            }
        }
        Err(error) => {
            log::error!("{error:?}");
            None
        }
    };
    log::trace!("{result:?}");

    Ok(result)
}

/// handle
/// URL: https://fullnode.devnet.aptoslabs.com/v1/accounts/{address}/resources
fn load_handel_link(
    profile: &ProfileConfig,
    address: &AccountAddress,
) -> Result<HashMap<AccountAddress, HandleRequest>> {
    let address_hex = address.to_hex_literal();

    let rest_url = profile
        .rest_url
        .as_ref()
        .ok_or_else(|| anyhow!("The rest_url in the profile is not specified"))?;
    let url = format!("{rest_url}/v1/accounts/{address_hex}/resources");

    let result = request_json_by_url::<Vec<MoveResource>>(&url)?
        .into_iter()
        .filter_map(|resource| {
            let index = Identifier::from_str("events").ok()?;
            resource.data.0.get(&index)?;

            let index = Identifier::from_str("tbl").ok()?;
            let table_handle = resource
                .data
                .0
                .get(&index)?
                .get("handle")?
                .as_str()?
                .to_string();
            let handle = AccountAddress::from_hex_literal(&table_handle).ok()?;
            let u256_address = format!(
                "{}::{}::U256",
                resource.typ.address.to_hex_literal(),
                resource.typ.module
            );

            Some((
                handle,
                HandleRequest {
                    url: Url::from_str(&format!("{rest_url}/v1/tables/{table_handle}/item"))
                        .ok()?,
                    key_type: u256_address.clone(),
                    value_type: u256_address,
                },
            ))
        })
        .collect();
    Ok(result)
}

/// Returns a list of modules with a bytecode
/// URL: https://fullnode.devnet.aptoslabs.com/v1/accounts/{address}/modules
fn load_modules(
    profile: &ProfileConfig,
    address: &AccountAddress,
) -> Result<BTreeMap<StateKey, Vec<u8>>> {
    let address_hex = address.to_hex_literal();

    let rest_url = profile
        .rest_url
        .as_ref()
        .ok_or_else(|| anyhow!("The rest_url in the profile is not specified"))?;
    let url = format!("{rest_url}/v1/accounts/{address_hex}/modules");

    Ok(request_bcs_by_url::<MoveModuleId>(&url)?
        .into_iter()
        .map(|(module_id, bytecode)| {
            let acc = StateKey::AccessPath(AccessPath::code_access_path(ModuleId::new(
                module_id.address,
                module_id.name,
            )));
            (acc, bytecode)
        })
        .collect())
}

/// Returns a list resources with a bytecode
/// URL: https://fullnode.devnet.aptoslabs.com/v1/accounts/{address}/resources
fn load_resource(
    profile: &ProfileConfig,
    address: &AccountAddress,
) -> Result<BTreeMap<StateKey, Vec<u8>>> {
    let address_hex = address.to_hex_literal();

    let rest_url = profile
        .rest_url
        .as_ref()
        .ok_or_else(|| anyhow!("The rest_url in the profile is not specified"))?;
    let url = format!("{rest_url}/v1/accounts/{address_hex}/resources");

    let list: BTreeMap<StateKey, Vec<u8>> = request_bcs_by_url::<StructTag>(&url)?
        .into_iter()
        .map(|(st, value)| {
            let rs = ResourceKey::new(*address, st);
            let acc = StateKey::AccessPath(AccessPath::resource_access_path(rs));
            (acc, value)
        })
        .collect();

    Ok(list)
}

fn request_json_by_url<T>(url: &str) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "accept",
        header::HeaderValue::from_static("application/json"),
    );

    let response = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()?
        .get(url)
        .send()?
        .json()?;
    Ok(response)
}

fn request_bcs_by_url<T>(url: &str) -> Result<BTreeMap<T, Vec<u8>>>
where
    T: serde::de::DeserializeOwned + std::cmp::Ord,
{
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "accept",
        header::HeaderValue::from_static("application/x-bcs"),
    );

    let response = reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()?
        .get(url)
        .send()?
        .bytes()?;
    Ok(bcs::from_bytes(response.as_ref())?)
}
