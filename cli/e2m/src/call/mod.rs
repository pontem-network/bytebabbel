use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, ensure, Result};
use aptos::common::types::CliCommand;
use clap::{Parser, ValueEnum};
use ethabi::{Contract, ParamType};
use move_core_types::account_address::AccountAddress;

pub(crate) mod function_id;

use crate::call::function_id::FunctionId;
use crate::profile::profile_to_address;
use crate::{wait, Cmd};
use eth::abi::call::to_token;
use eth::Flags;
use move_executor::load::LoadRemoteData;
use move_executor::solidity::FromSolidity;
use move_executor::{MoveExecutor, MoveExecutorInstance};

#[derive(Parser, Debug)]
pub struct CmdCall {
    /// Function name as `<ADDRESS>::<MODULE_ID>::<FUNCTION_NAME>`
    ///
    /// Example:
    /// `0x842ed41fad9640a2ad08fdd7d3e4f7f505319aac7d67e1c0dd6a7cce8732c7e3::message::set_message`
    #[clap(
        short,
        long = "function-id",
        value_name = "ADDRESS::MODULE_ID::FUNCTION_NAME",
        display_order = 1
    )]
    function_id: FunctionId,

    /// Arguments combined with their type separated by spaces.
    ///             
    /// Supported types [u8, u64, u128, bool, hex, string, address, raw]
    ///             
    /// Example: `address:0x1 bool:true u8:0`
    #[clap(
        short,
        long,
        multiple = true,
        value_name = "type1:value1 type2:value2 ...",
        display_order = 2
    )]
    args: Vec<String>,

    /// Use native "move" types for input and output values
    #[clap(long, display_order = 3)]
    native_type: bool,

    /// [node] - Call a remote contract on a node
    ///
    /// [local] - Call a remote contract locally and display the return value
    ///
    /// [local-source] - Call a local contract with remote resources and display the return value
    ///
    /// [vm] - Call a local contract and display the return value
    #[clap(
        long = "how",
        value_enum,
        default_value_t = HowToCall::Node,
        display_order = 10
    )]
    how_to_call: HowToCall,

    /// Path to converted project or sol file
    #[clap(long = "path", value_parser, value_name = "PATH", display_order = 11)]
    path_to_convert: Option<PathBuf>,

    #[clap(long = "profile", default_value = "default")]
    profile_name: String,

    #[clap(flatten)]
    transaction_flags: crate::txflags::TransactionFlags,
}

impl Cmd for CmdCall {
    fn execute(&self) -> Result<String> {
        match self.how_to_call {
            HowToCall::Node => self.call_remote_node(),
            HowToCall::Local | HowToCall::LocalSource | HowToCall::VM => self.call_local(),
        }
    }
}

impl CmdCall {
    fn call_remote_node(&self) -> Result<String> {
        use aptos::move_tool::RunFunction;

        let mut move_run_args = [
            "subcommand",
            "--profile",
            &self.profile_name,
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
            let args = if !self.native_type {
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

    fn call_local(&self) -> Result<String> {
        let path = self.path_to_convert.as_deref().ok_or_else(|| {
            anyhow!("Specify the path to the converted project or sol file. `--path <PATH/TO>`")
        })?;
        ensure!(path.exists(), "{path:?} not exist");

        let profile = move_executor::profile::load_profile(&self.profile_name)?;
        let signer_address = move_executor::profile::profile_to_address(&profile)?;
        let signer_address_hex = signer_address.to_hex_literal();

        let flag = if self.native_type {
            Flags::native_interface()
        } else {
            Flags::default()
        };

        let mut vm = if path.is_file() {
            ensure!(
                ext(path).as_deref() == Some("sol"),
                "Specify the path to sol file. Invalid file path {path:?}"
            );
            MoveExecutor::from_sol(path, signer_address, "", flag)?
        } else {
            let abi = find_abi(path)?;
            let mut vm = MoveExecutor::new(abi, flag, MoveExecutorInstance::Aptos);
            vm.deploy(&signer_address_hex, find_mv(path)?)?;
            vm
        };

        match self.how_to_call {
            HowToCall::Local => {
                let function_address = AccountAddress::from_hex_literal(&self.function_id.address)?;

                println!("Loading remote modules and resources for {function_address:?}");
                vm.load_all(&profile, &function_address)?;

                if function_address != signer_address {
                    println!("Loading remote modules and resources for {signer_address_hex:?}");
                    vm.load_all(&profile, &signer_address)?;
                }
            }
            HowToCall::LocalSource => {
                let function_address = AccountAddress::from_hex_literal(&self.function_id.address)?;

                println!("Loading remote modules and resources for {function_address:?}");
                vm.load_resources(&profile, &function_address)?;

                if function_address != signer_address {
                    println!("Loading remote modules and resources for {signer_address_hex:?}");
                    vm.load_resources(&profile, &signer_address)?;
                }
            }
            HowToCall::VM => {
                let fn_constructor = format!(
                    "{}::{}::constructor",
                    self.function_id.address, self.function_id.module
                );

                println!("LOCAL RUN: {fn_constructor}");
                vm.run(&fn_constructor, &signer_address_hex, None).unwrap();
            }
            HowToCall::Node => unreachable!(),
        }

        let fn_string = self.function_id.to_string();
        let args = self
            .args
            .iter()
            .map(|val| {
                val.split_once(':')
                    .map_or_else(|| val.as_str(), |(.., val)| val)
            })
            .map(|val| val.trim())
            .collect::<Vec<&str>>()
            .join(" ");

        println!("LOCAL RUN: {fn_string}({})", self.args.join(", ").trim());
        let res = vm
            .run(&fn_string, &signer_address_hex, Some(args.as_str()))?
            .to_result_str();

        Ok(res)
    }
}

#[derive(Debug, Clone, ValueEnum)]
enum HowToCall {
    /// Call a remote contract on a node
    Node,
    /// Call a remote contract locally and display the return value
    Local,
    /// [local-source] - call a local contract with remote resources and display the return value
    LocalSource,
    /// [vm] - call a local contract and display the return value
    VM,
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

fn ext(path: &Path) -> Option<String> {
    Some(path.extension()?.to_string_lossy().to_string())
}

fn find_by_ext(root_path: &Path, ext_str: &str) -> Option<PathBuf> {
    root_path
        .read_dir()
        .ok()?
        .filter_map(|item| item.ok())
        .map(|item| item.path())
        .find(|path| ext(path).as_deref() == Some(ext_str))
}

fn find_abi(path: &Path) -> Result<Contract> {
    let abi_path =
        find_by_ext(path, "abi").ok_or_else(|| anyhow!("Expected <FILE>.abi in the {path:?}"))?;
    let abi: Contract = serde_json::from_str(&fs::read_to_string(&abi_path)?)?;
    Ok(abi)
}

fn find_mv(path: &Path) -> Result<Vec<u8>> {
    let mv_binarycode =
        find_by_ext(path, "mv").ok_or_else(|| anyhow!("Expected <FILE>.mv in the {path:?}"))?;
    let mv = fs::read(&mv_binarycode)?;
    Ok(mv)
}
