use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as cli;

use anyhow::{anyhow, bail, Result};
use clap::Parser;
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
    pub(crate) deploy_flags: flags::DeployFlags,

    #[clap(flatten)]
    translation_flags: flags::TranslationFlags,
}

impl Cmd for CmdConvert {
    fn execute(&self) -> Result<String> {
        let result = self.convert()?;

        #[cfg(feature = "deploy")]
        if self.deploy_flags.deploy {
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
        let paths = path_to_abibin(&self.path)?;
        let mv_path = self
            .output_path
            .clone()
            .unwrap_or_else(|| {
                let filename = path_to_filename(&paths.abi).unwrap();
                PathBuf::from("./").join(filename)
            })
            .with_extension("mv");

        let interface_path = self
            .output_path
            .clone()
            .unwrap_or_else(|| {
                let filename = path_to_filename(&paths.abi).unwrap();
                PathBuf::from("./").join(filename)
            })
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
            .unwrap_or(path_to_filename(&paths.abi)?);

        let abi_content = fs::read_to_string(&paths.abi)?;
        let eth_content = fs::read_to_string(&paths.bin)?;

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
        let mv = translate(&eth_content, &abi_content, cfg)?;
        fs::write(&mv_path, &mv.bytecode)?;
        save_interface(
            &interface_path,
            &mv,
            self.translation_flags.interface_package,
        )?;

        paths.delete_tmp_dir();

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
fn path_to_abibin(path: &Path) -> Result<SolPaths> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| anyhow!("solidity file was expected {path:?}\nPath: "))?;

    match ext {
        "sol" => compile_sol(path),
        "bin" | "abi" => find_abibin(path),
        _ => bail!("A file with the extension bin, abi or sol was expected.\nPath: {path:?}"),
    }
}

/// Compile the sol file and return the paths
///     return: (abi path, bin path)
fn compile_sol(path: &Path) -> Result<SolPaths> {
    todo!();

    let path = path.canonicalize()?;

    if !check_solc() {
        bail!("solc command was not found.\n\
        Please install solc on your computer. See: https://docs.soliditylang.org/en/develop/installing-solidity.html")
    }

    let tmp_folder = tempfile::TempDir::new()?.into_path();
    log::debug!("tmp_dir: {tmp_folder:?}");

    let result = cli::new("solc")
        .args([
            "-o",
            &tmp_folder.to_string_lossy(),
            "--optimize-runs=0",
            "--abi",
            "--bin",
            "--ast-compact-json",
            "--asm",
            path.to_string_lossy().to_string().as_str(),
        ])
        .output()?;
    let result = output_to_result(result)?;
    log::info!("{result}");

    let files: HashMap<String, PathBuf> = fs::read_dir(&tmp_folder)?
        .filter_map(|item| item.ok())
        .map(|item| item.path())
        .filter(|item| item.is_file())
        .filter_map(|item| {
            let ext = item
                .extension()
                .map(|ext| ext.to_string_lossy().to_string())?;
            Some((ext, item))
        })
        .filter(|(name, _)| ["abi", "bin"].contains(&name.as_str()))
        .collect();
    log::debug!("finded: {files:?}");

    Ok(SolPaths {
        abi: files["abi"].to_owned(),
        bin: files["bin"].to_owned(),
        tmp_dir: Some(tmp_folder),
    })
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

fn find_abibin(path: &Path) -> Result<SolPaths> {
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

    Ok(SolPaths {
        abi: abi_path,
        bin: bin_path,
        tmp_dir: None,
    })
}

#[derive(Debug)]
struct SolPaths {
    abi: PathBuf,
    bin: PathBuf,
    tmp_dir: Option<PathBuf>,
}

impl SolPaths {
    pub fn delete_tmp_dir(&self) {
        if let Some(path) = &self.tmp_dir {
            log::debug!("Deleting a temporary directory {path:?}");
            if !path.exists() {
                log::error!("The path does not exist {path:?}");
                return;
            }
            if !path.is_dir() {
                log::error!("The path is not a directory {path:?}");
                return;
            }
            if let Err(err) = fs::remove_dir_all(path) {
                log::error!("{err:?}");
            }
            log::debug!("Temporary directory deleted");
        }
    }
}

pub struct ResultConvert {
    pub mv_path: PathBuf,
    pub move_path: PathBuf,
    pub module_name: String,
    pub address: AccountAddress,
}
