use std::str::FromStr;
use std::string::{String, ToString};

use crate::profile::ProfileValue;
use anyhow::{anyhow, Error};

#[derive(Debug)]
pub(crate) struct ResourcePath {
    address: String,
    module: String,
    structure: String,
    pub(crate) field: Option<String>,
}

impl ToString for ResourcePath {
    fn to_string(&self) -> String {
        format!(
            "{address}::{module}::{structure}{field}",
            address = &self.address,
            module = &self.module,
            structure = &self.structure,
            field = self
                .field
                .as_ref()
                .map(|v| format!("/{v}"))
                .unwrap_or_default()
        )
    }
}

impl FromStr for ResourcePath {
    type Err = Error;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut parts = value.split("::").map(|v| v.trim());

        let err_format = || {
            anyhow!("`--resource_path` the `<ADDRESS>::<MODULE>::<STRUCTURE>` format was expected")
        };

        let address = parts.next().ok_or_else(err_format)?.to_string();
        let address_hex = ProfileValue::from_str(&address)?
            .to_address()?
            .to_hex_literal();

        Ok(ResourcePath {
            address: address_hex,
            module: parts.next().ok_or_else(err_format)?.to_string(),
            structure: parts.next().ok_or_else(err_format)?.to_string(),
            field: parts.next().map(String::from),
        })
    }
}
