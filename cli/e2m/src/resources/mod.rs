use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::string::ToString;

use anyhow::{anyhow, ensure, Result};
use aptos::account::list::ListAccount;
use aptos::common::types::{CliCommand, ProfileOptions, RestOptions};
use aptos_types::account_address::AccountAddress;
use clap::Parser;
use reqwest::Url;
use serde_json::Value;

use test_infra::color::{bold, font_yellow};

pub mod decode;
pub mod query;
pub mod resource_path;

use crate::profile::ProfileValue;
use crate::{wait, Cmd};
use decode::{decode, decode_by_abi};
use query::ListQuery;
use resource_path::ResourcePath;

#[derive(Parser, Debug)]
pub struct CmdResources {
    /// Address of the account you want to list resources/modules for
    #[clap(long, parse(try_from_str = aptos::common::types::load_account_arg))]
    account: Option<AccountAddress>,

    /// Type of items to list: [balance, resources, modules, resource, events]
    #[clap(long, default_value_t = ListQuery::Resources)]
    query: ListQuery,

    #[clap(flatten)]
    rest_options: RestOptions,

    #[clap(flatten)]
    profile_options: ProfileOptions,

    /// Query `<ADDRESS>::<MODULE_ID>::<FUNCTION_NAME>(::<FIELD_NAME>)?`
    ///
    /// Example:
    /// `0x842ed41fad9640a2ad08fdd7d3e4f7f505319aac7d67e1c0dd6a7cce8732c7e3::ModuleName::StuctureName`
    /// `0x842ed41fad9640a2ad08fdd7d3e4f7f505319aac7d67e1c0dd6a7cce8732c7e3::ModuleName::StuctureName::field_name`
    #[clap(short, long)]
    resource_path: Option<ResourcePath>,

    /// Types for decoding.
    /// Used for decoding EVENTS
    #[clap(long, multiple = true)]
    decode_types: Vec<String>,

    /// Path to abi for decoding events
    #[clap(long, value_parser)]
    abi: Option<PathBuf>,

    /// Max number of events to retrieve.
    #[clap(long, default_value = "10", value_parser)]
    limit: u64,

    /// Starting sequence number of events.
    #[clap(long, default_value = "0", value_parser)]
    start: u64,
}

impl Cmd for CmdResources {
    fn execute(&self) -> Result<String> {
        match self.query {
            // The request will be made via the "aptos cli`
            ListQuery::Balance | ListQuery::Modules | ListQuery::Resources => self.parent(),

            ListQuery::Resource => self.resource(),
            ListQuery::Events => self.events(),
        }
    }
}

impl CmdResources {
    /// run $ aptos account list ..
    fn parent(&self) -> Result<String> {
        show_ignored_message(self.resource_path.is_some(), "--resource_path");
        show_ignored_message(!self.decode_types.is_empty(), "--decode-types");

        let mut aptos_accout_list_args = vec![
            "subcommand".to_string(),
            "--query".to_string(),
            self.query.to_string(),
            "--url".to_string(),
            self.rest_options.url(&self.profile_options)?.to_string(),
        ];

        if let Some(profile) = self.profile_options.profile.as_ref() {
            aptos_accout_list_args.extend(["--profile".to_string(), profile.clone()]);
        }

        if let Some(account) = &self.account {
            aptos_accout_list_args.extend(["--account".to_string(), account.to_hex()]);
        }

        let aptos_run_cli: ListAccount = ListAccount::try_parse_from(&aptos_accout_list_args)
            .map_err(|err| anyhow!("Invalid parameter. {err}"))?;

        let result = wait(aptos_run_cli.execute())?;
        Ok(serde_json::to_string_pretty(&serde_json::to_value(
            &result,
        )?)?)
    }

    // = = =

    fn resource(&self) -> Result<String> {
        let resource_path: &ResourcePath = self.resource_path_must()?;
        ensure!(
            resource_path.field.is_none(),
            "`--resource_path` the `<ADDRESS>::<MODULE>::<STRUCTURE>` format was expected. \n\
            Passed parameter {}",
            resource_path.to_string()
        );
        self.load_as_str()
    }

    fn events(&self) -> Result<String> {
        let resource_path = self.resource_path_must()?;
        ensure!(
            resource_path.field.is_some(),
            "`--resource_path` the `<ADDRESS>::<MODULE>::<STRUCTURE>::<FIELD>` format was expected. \n\
            Passed parameter {}",
            resource_path.to_string()
        );
        self.load_as_str()
    }
}

impl CmdResources {
    fn url(&self) -> Result<Url> {
        let resource_path: &ResourcePath = self.resource_path_must()?;

        let request_url_base = self
            .rest_options
            .url(&self.profile_options)?
            .as_str()
            .to_string();

        let account_hex = match self.account {
            None => ProfileValue::from_str(
                self.profile_options.profile.as_deref().unwrap_or("default"),
            )?
            .to_address()?,
            Some(address) => address,
        }
        .to_hex();

        let path = resource_path.to_string();

        let url_string = match self.query {
            ListQuery::Balance | ListQuery::Modules | ListQuery::Resources => unreachable!(),
            ListQuery::Resource => {
                format!("{request_url_base}/accounts/{account_hex}/resource/{path}")
            }
            ListQuery::Events => {
                format!("{request_url_base}/accounts/{account_hex}/events/{path}?limit={limit}&start={start}", limit = self.limit, start = self.start)
            }
        };
        let url = Url::from_str(&url_string)?;
        Ok(url)
    }

    fn resource_path_must(&self) -> Result<&ResourcePath> {
        self.resource_path.as_ref().ok_or_else(|| {
            anyhow!(
                "Specify which resource needs to be displayed: {}",
                bold("--resource_path")
            )
        })
    }

    fn load(&self) -> Result<Value> {
        let url = self.url()?;
        log::debug!("url: {url}");

        let mut response: Value = reqwest::blocking::get(url)?.json()?;

        if let Some(path) = &self.abi {
            let abi_string = serde_json::from_str(&fs::read_to_string(path)?)?;
            decode_by_abi(&mut response, &abi_string);
        }

        if !self.decode_types.is_empty() {
            response = decode(response, &self.decode_types)?;
        }
        log::debug!("response: {response:?}");

        Ok(response)
    }

    fn load_as_str(&self) -> Result<String> {
        let json = self.load()?;
        let json_string = serde_json::to_string_pretty(&json)?;
        log::debug!("json: {json_string}");

        Ok(json_string)
    }
}

#[inline]
fn show_ignored_message(show: bool, name_args: &str) {
    if show {
        println!(
            "[{head}] The {arg} parameters were ignored. Use it with {event}.",
            head = font_yellow("ignored"),
            arg = bold(name_args),
            event = bold("--query events"),
        )
    }
}
