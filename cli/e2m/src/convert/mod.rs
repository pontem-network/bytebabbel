use std::fs;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::process::Command as cli;
use std::sync::Arc;

use anyhow::{anyhow, bail, Result};
use clap::Parser;
use eth::compile::{Evm, EvmPack};
use move_core_types::account_address::AccountAddress;

use crate::{profile, Cmd};
use translator::{translate, Flags, Target};

#[cfg(feature = "deploy")]
mod deploy;
pub mod flags;

#[derive(Parser, Debug)]
pub struct CmdConvert {
    /// Path to the file. Specify the path to sol file or abi|bin.
    #[clap(value_parser)]
    path: PathBuf,

    /// Where to save the converted move binary code
    #[clap(short, long = "output", display_order = 3, value_parser)]
    output_path: Option<PathBuf>,

    /// The name of the Move module. If not specified, the name will be taken from the abi path
    #[clap(long = "module", display_order = 4, value_parser)]
    move_module_name: Option<String>,

    /// Profile name or address. The address must start with "0x". Needed for the module address
    #[clap(
        long = "profile",
        display_order = 5,
        short = 'p',
        default_value = "default",
        value_parser
    )]
    pub(crate) profile_or_address: profile::ProfileValue,

    /// Parameters for initialization
    #[clap(long = "args", short = 'a', default_value = "")]
    init_args: String,

    #[cfg(feature = "deploy")]
    #[clap(flatten)]
    pub(crate) deploy_flags: crate::txflags::TransactionFlags,

    /// Publishes the modules in a Move package to the Aptos blockchain
    #[cfg(feature = "deploy")]
    #[clap(long = "deploy", short = 'd', value_parser)]
    pub deploy: bool,

    #[clap(flatten)]
    translation_flags: flags::ConvertFlags,
}

impl Cmd for CmdConvert {
    fn execute(&self) -> Result<String> {
        let result = self.convert()?;

        #[cfg(feature = "deploy")]
        if self.deploy {
            return self.publish(&result);
        }

        self.translation_flags.check()?;

        Ok(format!(
            "{}\n{}",
            result.mv_path.to_string_lossy(),
            result.move_path.to_string_lossy()
        ))
    }
}

impl CmdConvert {
    pub fn convert(&self) -> Result<ResultConvert> {
        let pack = path_to_abibin(&self.path)?;
        let mv_path = self
            .output_path
            .clone()
            .unwrap_or_else(|| PathBuf::from("./").join(pack.name()))
            .with_extension("mv");

        let interface_path = self
            .output_path
            .clone()
            .unwrap_or_else(|| PathBuf::from("./").join(pack.name()))
            .with_extension("move");

        let address = self.profile_or_address.to_address()?;
        let address_str = format!("0x{}", &address);

        let mut init_args = self.init_args.clone();
        while let Some(pos) = init_args.to_lowercase().find("self") {
            init_args.replace_range(pos..pos + 4, &address_str);
        }

        let module_name = self
            .move_module_name
            .clone()
            .unwrap_or_else(|| pack.name().to_string());

        let cfg = translator::Config {
            contract_addr: address,
            name: &module_name,
            initialization_args: &init_args,
            flags: Flags {
                native_input: self.translation_flags.native_input,
                native_output: self.translation_flags.native_output,
                hidden_output: self.translation_flags.hide_output,
                u128_io: self.translation_flags.u128_io,
                package_interface: self.translation_flags.interface_package,
            },
        };
        let mv = translate(pack.bin_contract(), pack.abi_str(), cfg)?;
        fs::write(&mv_path, &mv.bytecode)?;
        save_interface(
            &interface_path,
            &mv,
            self.translation_flags.interface_package,
        )?;

        let move_path = if self.translation_flags.interface_package {
            interface_path.with_extension("")
        } else {
            interface_path
        };

        Ok(ResultConvert {
            mv_path,
            move_path,
            module_name,
            address,
        })
    }
}

fn save_interface(path: &Path, target: &Target, save_as_package: bool) -> Result<()> {
    if path.exists() {
        if path.is_file() {
            fs::remove_file(path)?;
        } else {
            fs::remove_dir_all(path)?;
        }
    }

    if save_as_package {
        let base_dir = path.with_extension("");
        let name = base_dir
            .file_name()
            .ok_or_else(|| anyhow!("Invalid path"))?;

        fs::create_dir_all(&base_dir)?;
        fs::write(base_dir.join("Move.toml"), &target.manifest)?;
        let sources = base_dir.join("sources");
        fs::create_dir_all(&sources)?;
        fs::write(sources.join(name).with_extension("move"), &target.interface)?;
    } else {
        fs::write(path, &target.interface)?;
    }
    Ok(())
}

fn path_to_filename(path: &Path) -> Result<String> {
    let name = path
        .with_extension("")
        .file_name()
        .ok_or_else(|| anyhow!("incorrect file path: {path:?}"))?
        .to_string_lossy()
        .to_string();
    Ok(name)
}

/// Convert the passed path to "abi" and "bin" path.
///     sol - compiled into "bin" and "abi" and stored in a temporary directory
///     bin - searches next to "abi" with the same name and returns paths to them
///     abi - searches next to "bin" with the same name and returns paths to them
fn path_to_abibin(path: &Path) -> Result<EvmPack> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| anyhow!("solidity file was expected {path:?}\nPath: "))?;

    match ext {
        "sol" => {
            if !check_solc() {
                bail!("solc command was not found.\n\
                    Please install solc on your computer. See: https://docs.soliditylang.org/en/develop/installing-solidity.html")
            }
            eth::compile::build_sol_by_path(path)
        }
        "bin" | "abi" => find_abibin(path),
        _ => bail!("A file with the extension bin, abi or sol was expected.\nPath: {path:?}"),
    }
}

/// Checking whether "solc" is installed on this computer
fn check_solc() -> bool {
    let output = match cli::new("solc").arg("--version").output() {
        Ok(r) => r,
        Err(err) => {
            log::error!("{err}");
            return false;
        }
    };

    match output_to_result(output) {
        Ok(version) => {
            log::info!("{version}");
            true
        }
        Err(error) => {
            log::error!("{error}");
            false
        }
    }
}

fn output_to_result(output: std::process::Output) -> Result<String> {
    if !output.status.success() {
        bail!(
            "{error}",
            error = String::from_utf8(output.stderr).unwrap_or_default()
        )
    }

    Ok(String::from_utf8(output.stdout).unwrap_or_default())
}

fn find_abibin(path: &Path) -> Result<EvmPack> {
    let filename = path_to_filename(path)?;
    let dir = path
        .parent()
        .ok_or_else(|| anyhow!("Parent directory not found.\nPath: {path:?}"))?;

    let abi_path = dir.join(format!("{filename}.abi"));
    if !abi_path.exists() || !abi_path.is_file() {
        bail!("Couldn't find abi.\nPath:{abi_path:?}");
    }

    let bin_path = dir.join(format!("{filename}.bin"));
    if !bin_path.exists() || !bin_path.is_file() {
        bail!("Couldn't find bin.\nPath:{bin_path:?}");
    }

    let bin = read_to_string(&bin_path)?;
    let abi = read_to_string(&abi_path)?;

    Ok(EvmPack::from((
        Evm {
            name: Arc::new(filename),
            bin: Arc::new(bin),
            abi: Arc::new(abi),
        },
        Vec::new(),
    )))
}

pub struct ResultConvert {
    pub mv_path: PathBuf,
    pub move_path: PathBuf,
    pub module_name: String,
    pub address: AccountAddress,
}
