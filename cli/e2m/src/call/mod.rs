use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use aptos::common::types::{CliCommand, ProfileOptions};
use clap::Parser;
use ethabi::{Contract, ParamType};

use eth::abi::call::to_token;
use eth::Flags;
use move_executor as me;
use move_executor::load::LoadRemoteData;
use move_executor::MoveExecutorInstance;

pub(crate) mod function_id;

use crate::call::function_id::FunctionId;
use crate::profile::profile_to_address;
use crate::{wait, Cmd};

#[derive(Parser, Debug)]
pub struct CmdCall {
    /// Arguments combined with their type separated by spaces.
    ///             
    /// Supported types [u8, u64, u128, bool, hex, string, address, raw]
    ///             
    /// Example: `address:0x1 bool:true u8:0`
    #[clap(short, long, multiple = true)]
    args: Vec<String>,

    /// Function name as `<ADDRESS>::<MODULE_ID>::<FUNCTION_NAME>`
    ///
    /// Example:
    /// `0x842ed41fad9640a2ad08fdd7d3e4f7f505319aac7d67e1c0dd6a7cce8732c7e3::message::set_message`
    #[clap(short, long = "function-id")]
    function_id: FunctionId,

    #[clap(flatten)]
    profile_options: ProfileOptions,

    /// Encode input params
    #[clap(long, default_value = "true")]
    encode: bool,

    #[clap(flatten)]
    transaction_flags: crate::txflags::TransactionFlags,

    /// Execute a locally remote contract
    #[clap(long)]
    local: bool,

    /// Path to abi for run view method
    #[clap(long = "abi", value_parser)]
    abi_path: Option<PathBuf>,
}

impl Cmd for CmdCall {
    fn execute(&self) -> Result<String> {
        use aptos::move_tool::RunFunction;

        if self.local {
            return self.try_local_run();
        }

        let profile_name = self.profile_options.profile_name().ok_or_else(|| {
            anyhow!(
                "For deploy, you need to specify the profile name. \n\n\
                    Example: \n\
                    $ e2m <path/to/file.sol> --profile <NameProfile>\n\n\
                    Create profile default: \n\
                    $ aptos init\n\n\
                    Create profile with name:\n\
                    $ aptos init --profile <NameProfile>"
            )
        })?;

        let mut move_run_args = [
            "subcommand",
            "--profile",
            profile_name,
            "--max-gas",
            &self.transaction_flags.max_gas.to_string(),
            "--assume-yes",
            "--function-id",
            &self.function_id.to_string(),
        ]
        .into_iter()
        .map(|item| item.to_string())
        .collect::<Vec<_>>();

        if !self.args.is_empty() {
            let args = if self.encode {
                format!("hex:{}", args_encode(&self.args)?)
            } else {
                self.args.join(" ")
            };
            move_run_args.push("--args".to_string());
            move_run_args.push(args);
        }

        let aptos_run_cli: RunFunction = RunFunction::try_parse_from(&move_run_args)
            .map_err(|err| anyhow!("Invalid parameter. {err}"))?;

        let result = wait(aptos_run_cli.execute())?;
        Ok(serde_json::to_string_pretty(&serde_json::to_value(
            result,
        )?)?)
    }
}

impl CmdCall {
    fn try_local_run(&self) -> Result<String> {
        let profile =
            me::profile::load_profile(self.profile_options.profile_name().unwrap_or("default"))?;

        let abi_path = self
            .abi_path
            .as_ref()
            .ok_or_else(|| anyhow!("`--abi` parameter is required"))?;

        let abi: Contract = serde_json::from_str(
            &fs::read_to_string(abi_path).map_err(|err| anyhow!("{abi_path:?} {err:?}"))?,
        )?;

        let flags = if self.encode {
            Flags::default()
        } else {
            Flags::native_interface()
        };

        let mut vm = me::MoveExecutor::new(abi, flags, MoveExecutorInstance::Aptos);
        vm.load_all(&profile)?;

        let res = vm
            .run(
                &self.function_id.to_string(),
                &me::profile::profile_to_address(&profile)?.to_hex_literal(),
                Some(""),
            )?
            .to_result_str();
        Ok(res)
    }
}

fn args_encode(args: &[String]) -> Result<String> {
    let eth_data = args
        .iter()
        .map(|row| {
            log::trace!("{}", row);
            let (type_str, val_str) = row
                .split_once(':')
                .ok_or_else(|| anyhow!("incorrect parameter. {row}"))?;
            let tp = ethabi::param_type::Reader::read(type_str)?;

            let mut val_str = val_str.to_string();
            if matches!(tp, ParamType::Address) {
                val_str = profile_to_address(&val_str)?.to_hex_literal();
            }

            let value = to_token(&(tp, &val_str))?;
            log::trace!("{:?}", &value);
            Ok(value)
        })
        .collect::<Result<Vec<_>>>()?;
    log::trace!("{:?}", &eth_data);

    let result = hex::encode(ethabi::encode(&eth_data));
    log::trace!("{}", &result);

    Ok(result)
}
