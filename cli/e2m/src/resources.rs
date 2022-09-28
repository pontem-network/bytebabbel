use std::{str::FromStr, string::ToString};

use anyhow::{anyhow, Result};
use clap::{ArgEnum, Parser};

use aptos::account::list::ListAccount;
use aptos::common::types::{CliCommand, ProfileOptions, RestOptions};
use aptos_types::account_address::AccountAddress;

use test_infra::color;

use crate::{wait, Cmd};

#[derive(Parser, Debug)]
pub struct CmdResources {
    /// Address of the account you want to list resources/modules for
    #[clap(long, parse(try_from_str=aptos::common::types::load_account_arg))]
    pub(crate) account: Option<AccountAddress>,

    /// Type of items to list: [balance, resources, modules, events]
    #[clap(long, default_value_t = ListQuery::Resources)]
    pub(crate) query: ListQuery,

    /// Query `<ADDRESS>::<MODULE_ID>::<FUNCTION_NAME>::<FIELD_NAME>`
    ///
    /// Example:
    /// `0x842ed41fad9640a2ad08fdd7d3e4f7f505319aac7d67e1c0dd6a7cce8732c7e3::ModuleName::StuctureName`
    /// `0x842ed41fad9640a2ad08fdd7d3e4f7f505319aac7d67e1c0dd6a7cce8732c7e3::ModuleName::StuctureName::field_name`
    #[clap(short, long)]
    pub(crate) resource_path: Option<String>,

    #[clap(flatten)]
    pub(crate) rest_options: RestOptions,

    #[clap(flatten)]
    pub(crate) profile_options: ProfileOptions,

    /// Types for decoding.
    /// Used for decoding EVENTS
    #[clap(long = "decode-types")]
    pub(crate) decode_types: Option<String>,
}

impl CmdResources {
    fn parent(&self) -> Result<String> {
        show_ignored_message(self.resource_path.is_some(), "--resource_path");
        show_ignored_message(self.decode_types.is_some(), "--decode-types");

        let profile = self.profile_options.profile.clone();

        let mut aptos_accout_list_args = vec![
            "subcommand".to_string(),
            "--query".to_string(),
            self.query.to_string(),
            "--url".to_string(),
            self.rest_options.url(&profile)?.to_string(),
            "--profile".to_string(),
            profile,
        ];

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

    fn events(&self) -> Result<String> {
        todo!()
    }
}

impl Cmd for CmdResources {
    fn execute(&self) -> Result<String> {
        match self.query {
            ListQuery::Balance | ListQuery::Modules | ListQuery::Resources => self.parent(),
            ListQuery::Events => self.events(),
        }
    }
}

#[derive(ArgEnum, Clone, Copy, Debug)]
pub enum ListQuery {
    Balance,
    Modules,
    Resources,
    Events,
}

impl ToString for ListQuery {
    fn to_string(&self) -> String {
        match self {
            ListQuery::Balance => "balance",
            ListQuery::Modules => "modules",
            ListQuery::Resources => "resources",
            ListQuery::Events => "events",
        }
        .to_string()
    }
}

impl FromStr for ListQuery {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "balance" => Ok(ListQuery::Balance),
            "modules" => Ok(ListQuery::Modules),
            "resources" => Ok(ListQuery::Resources),
            "events" => Ok(ListQuery::Events),
            _ => Err("Invalid query. Valid values are modules, resources"),
        }
    }
}

#[inline]
fn show_ignored_message(show: bool, name_args: &str) {
    if show {
        println!(
            "[{}] The {} parameters were ignored. Use it with {}.",
            color::font_yellow("ignored"),
            color::bold(name_args),
            color::bold("--query events"),
        )
    }
}
