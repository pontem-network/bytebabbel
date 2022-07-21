use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use clap::Parser;

use std::fs;
use std::process::Command as cli;

mod convert;
use crate::convert::Convert;

#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct Args {
    /// Path to the sol file
    #[clap(short, long, display_order = 1)]
    path: PathBuf,

    /// Where to save the converted file
    #[clap(short, long = "output", display_order = 2)]
    output_path: Option<PathBuf>,

    /// The name of the module. If not specified, the name will be taken from the abi path
    #[clap(long = "module", display_order = 3)]
    module_name: Option<String>,

    /// The address of the module.
    #[clap(long = "address", display_order = 4, default_value = "0x1")]
    module_address: String,

    /// Math backend.
    #[clap(long = "math", short = 'm', display_order = 5, default_value = "u128")]
    math_backend: String,

    #[clap(short, long, display_order = 6, value_parser)]
    trace: Option<bool>,
}

fn main() {
    match run() {
        Ok(path) => {
            println!("Saved in {path:?}");
        }
        Err(err) => {
            println!("Error: {err:?}");
        }
    }
}

fn run() -> Result<String> {
    let args: Args = Args::parse();
    inic_log::inic_of_log_configs(args.trace)?;

    let paths = compile_sol(&args.path)?;
    Convert::try_from((args, paths))?
        .create_mv()
        .map(|path| path.to_string_lossy().to_string())
}

/// Compile the sol file and return the paths
///     return: (abi path, bin path)
fn compile_sol(path: &Path) -> Result<(PathBuf, PathBuf)> {
    let path = path.canonicalize()?;

    if !check_solc() {
        bail!("solc command was not found.\n\
        Please install solc on your computer. See: https://docs.soliditylang.org/en/develop/installing-solidity.html")
    }

    let tmp_folder = temp_dir(&path)?;
    log::debug!("tmp_dir: {tmp_folder:?}");

    let result = cli::new("solc")
        .args(&[
            "-o",
            tmp_folder.to_string_lossy().to_string().as_ref(),
            "--bin",
            "--optimize-runs=0",
            "--abi",
            "--ast-compact-json",
            "--overwrite",
            "--error-recovery",
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

    Ok((files["abi"].to_owned(), files["bin"].to_owned()))
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

fn temp_dir(path: &Path) -> Result<PathBuf> {
    use sha2::{Digest, Sha256};

    let path_string = path.to_string_lossy().to_string();

    let mut hasher = Sha256::new();
    hasher.update(path_string.as_bytes());
    let hash = hasher.finalize();
    log::debug!("{hash:?}");

    let name_folder = base32::encode(base32::Alphabet::Crockford, &hash[0..16]);
    log::debug!("{name_folder}");

    let tmp = std::env::temp_dir().join(name_folder);
    if !tmp.exists() {
        fs::create_dir(&tmp)?;
    }

    Ok(tmp)
}
