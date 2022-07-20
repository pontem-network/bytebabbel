use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Error, Result};

use eth2move::Math;
use move_core_types::account_address::AccountAddress;

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
            eth2move::translate(self.address, &self.module_name, &eth, &abi, self.math)?;
        fs::write(&self.mv, move_bytecode)?;
        Ok(&self.mv)
    }
}

impl TryFrom<Args> for Convert {
    type Error = Error;
    fn try_from(args: Args) -> std::result::Result<Self, Self::Error> {
        let p = to_canonicalize(&[&args.abi_path, &args.bin_path])?;
        let abi = p[0].to_owned();
        let bin = p[1].to_owned();

        let address = AccountAddress::from_hex_literal(&args.module_address)?;
        let module_name = args.module_name.unwrap_or(path_to_filename(&abi)?);

        Ok(Convert {
            mv: args.output_path.unwrap_or(abi.with_extension("mv")),
            abi,
            bin,
            math: args.math_backend.parse()?,
            address,
            module_name,
        })
    }
}

fn to_canonicalize(paths: &[&PathBuf]) -> Result<Vec<PathBuf>> {
    let result = paths
        .iter()
        .map(|path| {
            if !path.exists() {
                bail!("file not found: {path:?}");
            }
            Ok(path.canonicalize()?)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(result)
}

fn path_to_filename(path: &Path) -> Result<String> {
    let name = path
        .with_extension("")
        .file_name()
        .ok_or(anyhow!("incorrect file path: {path:?}"))?
        .to_string_lossy()
        .to_string();
    Ok(name)
}
