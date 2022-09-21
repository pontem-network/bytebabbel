use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::{fs, io};

use anyhow::{anyhow, ensure, Error, Result};
use ethabi::Contract;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha3::{Digest, Sha3_256};

use eth::bytecode::pre_processing::swarm::remove_swarm_hash;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evm {
    name: Arc<String>,
    bin: Arc<String>,
    abi: Arc<String>,
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

pub fn build_sol_by_path(path: &Path) -> Result<EvmPack> {
    let path = path.canonicalize()?;
    let sol_contetnt = fs::read_to_string(&path)?;

    let tmp_dir_path = std::env::temp_dir().join(sha_name(sol_contetnt.as_bytes()) + "0");
    let contract_compile_path = tmp_dir_path.join("compiled.json");

    if !tmp_dir_path.exists() || !contract_compile_path.exists() {
        fs::create_dir_all(&tmp_dir_path)?;

        let output = Command::new("solc")
            .current_dir(tmp_dir_path.as_path())
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
                        let abi_string =
                            if abi.as_array().map(|item| !item.is_empty()).unwrap_or(false) {
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

        let contract = EvmPack::from((r_contracts.remove(0), r_modules));
        let json: Value = serde_json::to_value(&contract)?;
        let contract_str_formated = serde_json::to_string_pretty(&json)?;
        fs::write(&contract_compile_path, &contract_str_formated)?;

        return Ok(contract);
    }

    let cont = fs::read_to_string(&contract_compile_path)?;
    let contract: EvmPack = serde_json::from_str(&cont)?;

    Ok(contract)
}

pub fn build_sol(sol: &[u8]) -> Result<Evm, Error> {
    let tmp_dir = std::env::temp_dir().join(sha_name(sol));
    if !tmp_dir.exists() || fs::read_dir(&tmp_dir)?.count() != 2 {
        fs::create_dir_all(&tmp_dir)?;

        let contract = tmp_dir.join("contract.sol");
        fs::write(&contract, sol)?;

        let output = Command::new("solc")
            .current_dir(tmp_dir.as_path())
            .arg("-o")
            .arg(tmp_dir.as_path())
            .arg("--bin")
            .arg("--abi")
            .arg(contract.as_path())
            .output()?;
        io::stdout().write_all(&output.stdout)?;
        io::stderr().write_all(&output.stderr)?;
        ensure!(
            output.status.success(),
            "Compilation error:\n{}",
            String::from_utf8(output.stderr).unwrap_or_default()
        );
        fs::remove_file(&contract)?;
    }

    let dir = fs::read_dir(tmp_dir)?;
    let dir = dir.into_iter().collect::<Result<Vec<_>, _>>()?;
    ensure!(dir.len() == 2, "Expected 2 files in the output directory");

    let ext = dir[0]
        .path()
        .extension()
        .map(|ext| ext.to_string_lossy().to_string())
        .unwrap_or_default();
    let (bin, abi) = if ext == "bin" {
        (
            fs::read_to_string(dir[0].path())?,
            fs::read_to_string(dir[1].path())?,
        )
    } else {
        (
            fs::read_to_string(dir[1].path())?,
            fs::read_to_string(dir[0].path())?,
        )
    };

    let mut entry_path = dir[0].path();
    entry_path.set_extension("");
    let name = entry_path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default();

    Ok(Evm {
        name: Arc::new(name),
        abi: Arc::new(abi),
        bin: Arc::new(bin),
    })
}

fn sha_name(cont: &[u8]) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(cont);
    let value_hash = hasher.finalize();
    let hash: [u8; 32] = value_hash.as_slice().try_into().expect("Wrong length");
    format!(".c_{}", base32::encode(base32::Alphabet::Crockford, &hash))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
