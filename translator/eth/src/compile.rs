use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use anyhow::{anyhow, ensure, Result};
use ethabi::Contract;
use serde_json::Value;
use tempdir::TempDir;

use crate::bytecode::pre_processing::swarm::remove_swarm_hash;

#[derive(Debug, Clone)]
pub struct Evm {
    pub name: Arc<String>,
    pub bin: Arc<String>,
    pub abi: Arc<String>,
}

impl Evm {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn bin(&self) -> &str {
        &self.bin
    }

    pub fn abi(&self) -> &str {
        &self.abi
    }
}

pub fn build_sol<P: AsRef<Path>>(path: P) -> Result<EvmPack> {
    let path = path.as_ref().canonicalize()?;
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .ok_or_else(|| anyhow!("Invalid sol path:{:?}", path))?;

    let dir = TempDir::new("sol")?;
    if !dir.path().exists() {
        fs::create_dir_all(dir.path())?;
    }
    let contract = dir.path().join(name);
    fs::write(contract, fs::read_to_string(&path)?)?;

    let output = Command::new("solc")
        .current_dir(&dir)
        .args(["--combined-json", "abi,bin"])
        .arg(&path)
        .output()?;

    ensure!(
        output.status.success(),
        "Compilation error:\n{}",
        String::from_utf8(output.stderr).unwrap_or_default()
    );

    let json_output: Value = serde_json::from_str(&String::from_utf8(output.stdout)?)?;
    let list_evm = json_output
        .get("contracts")
        .and_then(|item| item.as_object())
        .map(|item| {
            item.iter()
                .filter_map(|(name, json)| {
                    let (_, name) = name.rsplit_once(':')?;

                    let abi = json.get("abi")?;
                    let abi_string = if abi.as_array().map(|item| !item.is_empty()).unwrap_or(false)
                    {
                        abi.to_string()
                    } else {
                        "".to_string()
                    };

                    let bin = json.get("bin")?.as_str().unwrap_or_default().to_string();
                    if bin.is_empty() {
                        return None;
                    }
                    Some(Evm {
                        name: Arc::new(name.to_string()),
                        abi: Arc::new(abi_string),
                        bin: Arc::new(bin),
                    })
                })
                .collect::<Vec<Evm>>()
        })
        .ok_or_else(|| anyhow!("Couldn't find a contract. {path:?}"))?;

    let (mut r_contracts, r_modules): (Vec<_>, Vec<_>) =
        list_evm.into_iter().partition(|item| !item.abi.is_empty());

    ensure!(
        r_contracts.len() == 1,
        "It was expected that there would be one contract in the file. {path:?}"
    );

    Ok(EvmPack::from((r_contracts.remove(0), r_modules)))
}

#[derive(Debug, Clone)]
pub struct EvmPack {
    contract: Evm,
    modules: Vec<Evm>,
}

impl EvmPack {
    pub fn contract(&self) -> &Evm {
        &self.contract
    }

    pub fn modules(&self) -> &Vec<Evm> {
        &self.modules
    }

    pub fn abi(&self) -> Result<Contract> {
        let abi = serde_json::from_str(self.contract.abi.as_str())?;
        Ok(abi)
    }

    pub fn bin_contract(&self) -> &str {
        self.contract.bin()
    }

    pub fn abi_str(&self) -> &str {
        self.contract.abi.as_str()
    }

    pub fn code(&self) -> Result<Vec<u8>> {
        let mut result: Vec<u8> = self
            .modules
            .iter()
            .map(|item| {
                let bin = hex::decode(item.bin())?;
                Ok(bin)
            })
            .collect::<Result<Vec<Vec<u8>>>>()?
            .into_iter()
            .flatten()
            .collect();
        let mut cont = hex::decode(self.contract.bin.as_str())?;
        result.append(&mut cont);

        Ok(result)
    }

    pub fn code_evm(&self) -> Result<Vec<u8>> {
        let mut result: Vec<u8> = self
            .modules
            .iter()
            .map(|item| evm_bytecode(hex::decode(item.bin()).unwrap()))
            .collect::<Vec<Vec<u8>>>()
            .into_iter()
            .flatten()
            .collect();
        let mut cont = evm_bytecode(hex::decode(self.contract.bin.as_str())?);
        result.append(&mut cont);

        Ok(result)
    }

    pub fn name(&self) -> &str {
        self.contract.name()
    }
}

impl From<(Evm, Vec<Evm>)> for EvmPack {
    fn from(data: (Evm, Vec<Evm>)) -> Self {
        EvmPack {
            contract: data.0,
            modules: data.1,
        }
    }
}

fn evm_bytecode(mut bytecode: Vec<u8>) -> Vec<u8> {
    remove_swarm_hash(&mut bytecode);
    bytecode
}
