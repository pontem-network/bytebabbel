use std::io::Write;
use std::process::Command;
use std::sync::Arc;
use std::{fs, io};

use anyhow::{ensure, Error};
use sha3::{Digest, Sha3_256};

#[derive(Debug, Clone)]
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

pub fn build_sol(sol: &[u8]) -> Result<Evm, Error> {
    let tmp_dir = std::env::temp_dir().join(sha_name(sol));
    if !tmp_dir.exists() {
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
    format!(".{}", base32::encode(base32::Alphabet::Crockford, &hash))
}
