use std::str::FromStr;

use anyhow::{anyhow, Result};
use aptos::common::types::CliCommand;
use clap::Parser;
use eth::abi::call::to_token;
use ethabi::ParamType;
use test_infra::color::font_green;

use crate::profile;
use crate::profile::ProfileValue;
use crate::Cmd;

use crate::wait;

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
    function_id: String,

    /// Profile name or address. The address must start with "0x". Needed for the module address
    #[clap(
        long = "profile",
        display_order = 5,
        short = 'p',
        default_value = "default",
        value_parser
    )]
    profile_or_address: profile::ProfileValue,

    /// Encode input params
    #[clap(long)]
    encode: bool,

    #[clap(flatten)]
    transaction_flags: crate::txflags::TransactionFlags,

    /// Display only. The request will not be sent to the aptos node
    #[clap(long)]
    sandbox: bool,
}

impl Cmd for CmdCall {
    fn execute(&self) -> Result<String> {
        use aptos::move_tool::RunFunction;

        let profile = self.profile_or_address.name_profile().map_err(|_| {
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
            profile,
            "--max-gas",
            &self.transaction_flags.max_gas.to_string(),
            "--assume-yes",
            "--function-id",
            &self.function_id,
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

        if self.sandbox {
            let cmd = format!(
                "{} aptos move run {}",
                font_green("Sandbox$"),
                move_run_args.join(" ")
            );
            return Ok(cmd);
        }
        let result = wait(aptos_run_cli.execute())?;
        Ok(serde_json::to_string_pretty(&serde_json::to_value(
            &result,
        )?)?)
    }
}

fn args_encode(args: &Vec<String>) -> Result<String> {
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
                val_str = ProfileValue::from_str(&val_str)
                    .and_then(|prof| prof.to_address())
                    .map(|add| hex::encode(add.as_ref()))
                    .unwrap_or(val_str.to_string());
                val_str = format!("0x{}", val_str);
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
