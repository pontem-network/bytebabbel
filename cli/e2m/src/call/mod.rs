use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, ensure, Result};
use aptos::common::types::CliCommand;
use clap::{Parser, ValueEnum};
use ethabi::Contract;
use itertools::Itertools;
use move_core_types::account_address::AccountAddress;

pub(crate) mod args;
pub(crate) mod function_id;

use crate::call::args::FunctionArgs;
use crate::call::function_id::FunctionId;
use crate::{wait, Cmd};
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

    /// Parameters for initialization.
    /// Required if a sol file is specified in the path
    ///
    /// Supported alias:
    ///
    /// self - account address from the profile
    ///
    /// <PROFILE_NAME> - account address from the profile
    ///
    /// Example:
    ///
    ///     `<PROFILE_NAME>` or `address:<PROFILE_NAME>`
    ///
    ///     `self` or `address:self`
    ///
    ///     `self true 0` or `address:self bool:true u8:0`
    #[clap(long, default_value = "")]
    init_args: Vec<String>,

    /// Use native "move" types for input and output values
    #[clap(long = "native", display_order = 3)]
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

    /// Load resources from addresses 0x1
    #[clap(long, display_order = 11, hide = true)]
    load_0x1: bool,

    /// Path to converted project or sol file
    #[clap(long = "path", value_parser, value_name = "PATH", display_order = 12)]
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
    /// [node] - Call a remote contract on a node
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
            move_run_args.push("--args".to_string());
            if !self.native_type {
                move_run_args.push(format!(
                    "hex:{}",
                    FunctionArgs::from(&self.args).args_encode()?
                ));
            } else {
                move_run_args.extend(self.args.clone());
            };
        }

        let aptos_run_cli: RunFunction = RunFunction::try_parse_from(&move_run_args)
            .map_err(|err| anyhow!("Invalid parameter. {err}"))?;

        let result = wait(aptos_run_cli.execute())?;
        Ok(serde_json::to_string_pretty(&serde_json::to_value(
            result,
        )?)?)
    }

    /// [local] - Call a remote contract locally and display the return value
    /// [local-source] - Call a local contract with remote resources and display the return value
    /// [vm] - Call a local contract and display the return value
    fn call_local(&self) -> Result<String> {
        log::trace!("call_local:");

        let path = self.path_to_convert.as_deref().ok_or_else(|| {
            anyhow!("Specify the path to the converted project or sol file. `--path <PATH/TO>`")
        })?;
        ensure!(path.exists(), "{path:?} not exist");
        log::trace!("{path:?}");

        let profile = move_executor::profile::load_profile(&self.profile_name)?;
        let signer_address = move_executor::profile::profile_to_address(&profile)?;
        let signer_address_hex = signer_address.to_hex_literal();
        log::trace!("{signer_address_hex}");

        let flag = if self.native_type {
            log::trace!("native");
            Flags::native_interface()
        } else {
            log::trace!("u256");
            Flags::default()
        };

        let mut vm = if path.is_file() && ext(path).as_deref() == Some("sol") {
            let init_args = FunctionArgs::from((self.profile_name.as_str(), &self.init_args))
                .value()
                .join(" ");

            MoveExecutor::from_sol(path, signer_address, &init_args, flag)?
        } else {
            let abi = find_abi(path)?;
            let mut vm = MoveExecutor::new(abi, flag, MoveExecutorInstance::Aptos);
            vm.deploy(&signer_address_hex, find_mv(path)?)?;
            vm
        };

        log::info!("{:?}", &self.how_to_call);
        match self.how_to_call {
            // [local] - Call a remote contract locally and display the return value
            HowToCall::Local => {
                let function_address = AccountAddress::from_hex_literal(&self.function_id.address)?;

                // loading resources from 0x1
                if self.load_0x1 {
                    println!("loading resources from 0x1");
                    vm.load_all(&profile, &AccountAddress::from_hex_literal("0x1")?)?;
                }

                println!("Loading remote modules and resources for {function_address:?}");
                vm.load_all(&profile, &function_address)?;

                if function_address != signer_address {
                    println!("Loading remote modules and resources for {signer_address_hex:?}");
                    vm.load_all(&profile, &signer_address)?;
                }
            }
            // [local-source] - Call a local contract with remote resources and display the return value
            HowToCall::LocalSource => {
                let fn_constructor = format!(
                    "{}::{}::constructor",
                    self.function_id.address, self.function_id.module
                );
                println!("LOCAL-SOURCE RUN: {fn_constructor}");
                vm.run(&fn_constructor, &signer_address_hex, None).unwrap();

                // loading resources from 0x1
                if self.load_0x1 {
                    println!("loading resources from 0x1");
                    vm.load_resources(&profile, &AccountAddress::from_hex_literal("0x1")?)?;
                }

                let function_address = AccountAddress::from_hex_literal(&self.function_id.address)?;
                println!("Loading remote resources for {function_address:?}");
                vm.load_resources(&profile, &function_address)?;

                if function_address != signer_address {
                    println!("Loading remote modules and resources for {signer_address_hex:?}");
                    vm.load_resources(&profile, &signer_address)?;
                }
            }
            // [vm] - Call a local contract and display the return value
            HowToCall::VM => {
                let fn_constructor = format!(
                    "{}::{}::constructor",
                    self.function_id.address, self.function_id.module
                );

                println!("VM RUN: {fn_constructor}");
                vm.run(&fn_constructor, &signer_address_hex, None).unwrap();
            }
            HowToCall::Node => unreachable!(),
        }

        let fn_string = self.function_id.to_string();
        let args = FunctionArgs::from(&self.args);

        println!("LOCAL RUN: {fn_string}({})", args.value().join(", ").trim());
        let res = vm
            .run(
                &fn_string,
                &signer_address_hex,
                Some(args.value().join(",").as_str()),
            )?
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

fn ext(path: &Path) -> Option<String> {
    Some(path.extension()?.to_string_lossy().to_string())
}

fn find_by_ext(root_path: &Path, ext_str: &str) -> Option<PathBuf> {
    if root_path.is_file() {
        let parent = root_path.parent()?;
        return find_by_ext(parent, ext_str);
    }

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
    let abi: Contract = serde_json::from_str(&fs::read_to_string(abi_path)?)?;
    Ok(abi)
}

fn find_mv(path: &Path) -> Result<Vec<u8>> {
    let mv_binarycode =
        find_by_ext(path, "mv").ok_or_else(|| anyhow!("Expected <FILE>.mv in the {path:?}"))?;
    let mv = fs::read(mv_binarycode)?;
    Ok(mv)
}
