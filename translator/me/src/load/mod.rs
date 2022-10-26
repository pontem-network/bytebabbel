use std::collections::BTreeMap;
use std::{collections::HashMap, str::FromStr};

use anyhow::{anyhow, Result};
use reqwest::{header, Url};
use serde_json::json;

use aptos::common::types::ProfileConfig;
use aptos_types::{access_path::AccessPath, state_store::state_key::StateKey};
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, ResourceKey, StructTag},
};

mod response;

use crate::{
    load::response::{MoveModuleId, MoveResource},
    profile_to_address,
    resolver::{print_access_path::AccessPathToString, HandleData},
    MoveExecutor,
};

pub trait LoadRemoteData {
    fn load_all(&mut self, profile: &ProfileConfig) -> Result<()> {
        self.load_modules(profile).and(self.load_resources(profile))
    }

    fn load_modules(&mut self, profile: &ProfileConfig) -> Result<()>;

    fn load_resources(&mut self, profile: &ProfileConfig) -> Result<()>;
}

impl LoadRemoteData for MoveExecutor {
    fn load_modules(&mut self, profile: &ProfileConfig) -> Result<()> {
        for (name, bytecode) in load_modules(profile)? {
            log::info!("loaded module: {}", name.print_string());
            self.resolver.state_data.insert(name, bytecode);
        }
        Ok(())
    }

    fn load_resources(&mut self, profile: &ProfileConfig) -> Result<()> {
        self.resolver.handler_data = load_handel_link(profile)?;

        for (name, bytecode) in load_resource(profile)? {
            log::info!("loaded resource: {}", name.print_string());
            self.resolver.state_data.insert(name, bytecode);
        }

        Ok(())
    }
}

/// https://fullnode.devnet.aptoslabs.com/v1/tables/{table_handle}/item
pub fn load_table_handle_u256(data: &HandleData, key: &Vec<u8>) -> Result<Option<Vec<u8>>> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Content-Type",
        header::HeaderValue::from_static("application/json"),
    );
    headers.insert(
        "accept",
        header::HeaderValue::from_static("application/x-bcs"),
    );
    let key_hex = hex::encode(key);
    let body = json!({
        "key_type":data.key_type,
        "value_type":data.value_type,
        "key":{
            "v0":&key_hex[0..16],
            "v1":&key_hex[16..32],
            "v2":&key_hex[32..48],
            "v3":&key_hex[48..64],
        }
    });

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
            match String::from_utf8(result.clone()) {
                Ok(msg) => {
                    log::error!("{msg}");
                    None
                }
                Err(_) => Some(result),
            }
        }
        Err(error) => {
            log::error!("{error:?}");
            None
        }
    };
    Ok(result)
}

/// handle
/// URL: https://fullnode.devnet.aptoslabs.com/v1/accounts/{address}/resources
fn load_handel_link(profile: &ProfileConfig) -> Result<HashMap<AccountAddress, HandleData>> {
    let address = profile_to_address(profile)?;
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
                HandleData {
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
fn load_modules(profile: &ProfileConfig) -> Result<BTreeMap<StateKey, Vec<u8>>> {
    let address = profile_to_address(profile)?;
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
fn load_resource(profile: &ProfileConfig) -> Result<BTreeMap<StateKey, Vec<u8>>> {
    let address = profile_to_address(profile)?;
    let address_hex = address.to_hex_literal();

    let rest_url = profile
        .rest_url
        .as_ref()
        .ok_or_else(|| anyhow!("The rest_url in the profile is not specified"))?;
    let url = format!("{rest_url}/v1/accounts/{address_hex}/resources");

    let list: BTreeMap<StateKey, Vec<u8>> = request_bcs_by_url::<StructTag>(&url)?
        .into_iter()
        .map(|(st, value)| {
            let rs = ResourceKey::new(address, st);
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
