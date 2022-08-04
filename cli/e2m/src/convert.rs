use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as cli;

use anyhow::{anyhow, bail, Result};
use clap::Parser;

use move_core_types::account_address::AccountAddress;

use translator::translate;

/// Converting the file solidity to binary code move
#[derive(Parser, Debug)]
pub struct Converting {
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
    profile_or_address: String,

    /// Math backend.
    #[clap(long = "math", short = 'm', default_value = "u128", value_parser)]
    math_backend: String,
}

impl Converting {
    pub fn execute(&self) -> Result<String> {
        let paths = path_to_abibin(&self.path)?;
        let output_path = self.output_path.clone().unwrap_or_else(|| {
            let filename = path_to_filename(&paths.abi).unwrap();
            PathBuf::from("./").join(filename).with_extension("mv")
        });

        let address = if self.profile_or_address.starts_with("0x") {
            AccountAddress::from_hex_literal(&self.profile_or_address)?
        } else {
            let aptos_account = aptos::common::types::CliConfig::load_profile(&self.profile_or_address)?
                .and_then(|profile| profile.account)
                .ok_or_else(|| {
                    anyhow!(
                        "Profile {:?} not found. To create a profile, use: $ e2m aptos init --profile <NAME>",
                        &self.profile_or_address
                    )
                })?;
            AccountAddress::from_bytes(&aptos_account.into_bytes())?
        };

        let module_name = self
            .move_module_name
            .clone()
            .unwrap_or(path_to_filename(&paths.abi)?);

        let abi_content = fs::read_to_string(&paths.abi)?;
        let eth_content = fs::read_to_string(&paths.bin)?;

        let move_bytecode: Vec<u8> = translate(
            address,
            &module_name,
            &eth_content,
            &abi_content,
            self.math_backend.parse()?,
        )?;
        fs::write(&output_path, move_bytecode)?;

        paths.delete_tmp_dir();

        Ok(output_path.to_string_lossy().to_string())
    }
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
    let path = path.canonicalize()?;

    if !check_solc() {
        bail!("solc command was not found.\n\
        Please install solc on your computer. See: https://docs.soliditylang.org/en/develop/installing-solidity.html")
    }

    let tmp_folder = tempfile::TempDir::new()?.into_path();
    log::debug!("tmp_dir: {tmp_folder:?}");

    let result = cli::new("solc")
        .args(&[
            "-o",
            tmp_folder.to_string_lossy().to_string().as_ref(),
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
