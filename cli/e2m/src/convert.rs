use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as cli;

use anyhow::{anyhow, bail, Error, Result};

use move_core_types::account_address::AccountAddress;
use translator::{translate, Math};

use crate::Args;

#[derive(Debug)]
pub struct Convert {
    abi: PathBuf,
    bin: PathBuf,
    mv: PathBuf,
    math: Math,
    address: AccountAddress,
    module_name: String,
}

impl Convert {
    pub fn create_mv(&self) -> Result<&Path> {
        let abi = fs::read_to_string(&self.abi)?;
        let eth = fs::read_to_string(&self.bin)?;

        let move_bytecode: Vec<u8> =
            translate(self.address, &self.module_name, &eth, &abi, self.math)?;
        fs::write(&self.mv, move_bytecode)?;
        Ok(&self.mv)
    }
}

impl TryFrom<Args> for Convert {
    type Error = Error;
    fn try_from(args: Args) -> std::result::Result<Self, Self::Error> {
        let (abi, bin) = path_to_abibin(&args.path)?;

        let address = AccountAddress::from_hex_literal(&args.move_module_address)?;
        let module_name = args.move_module_name.unwrap_or(path_to_filename(&abi)?);

        Ok(Convert {
            mv: args.output_path.unwrap_or_else(|| {
                let filename = path_to_filename(&abi).unwrap();
                PathBuf::from("./").join(filename).with_extension("mv")
            }),
            abi,
            bin,
            math: args.math_backend.parse()?,
            address,
            module_name,
        })
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
fn path_to_abibin(path: &Path) -> Result<(PathBuf, PathBuf)> {
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

/// Creating a temporary directory
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

fn find_abibin(path: &Path) -> Result<(PathBuf, PathBuf)> {
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

    Ok((abi_path, bin_path))
}
