use anyhow::Error;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

#[derive(Debug, Default)]
pub struct Abi {
    entries: HashMap<FunHash, AbiEntry>,
}

impl Abi {
    pub fn fun_hashes(&self) -> impl Iterator<Item = FunHash> + '_ {
        self.entries.iter().map(|(h, _)| *h)
    }
}

impl TryFrom<&str> for Abi {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let entries: Vec<AbiEntry> = serde_json::from_str(value)?;
        let entries = entries
            .into_iter()
            .map(|e| (FunHash::from(&e), e))
            .collect();
        Ok(Abi { entries })
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AbiEntry {
    inputs: Vec<AbiInput>,
    #[serde(default = "default_name")]
    name: String,
    #[serde(default = "default_outputs")]
    outputs: Vec<AbiOutput>,
    #[serde(alias = "stateMutability")]
    state_mutability: String,
    #[serde(alias = "type")]
    tp: String,
}

impl AbiEntry {
    pub fn signature(&self) -> String {
        format!(
            "{}({})",
            self.name,
            self.inputs.iter().map(|i| &i.tp).join(",")
        )
    }
}

fn default_outputs() -> Vec<AbiOutput> {
    vec![]
}

fn default_name() -> String {
    "".to_string()
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AbiInput {
    #[serde(alias = "internalType")]
    internal_type: String,
    name: String,
    #[serde(alias = "type")]
    tp: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AbiOutput {
    #[serde(alias = "internalType")]
    internal_type: String,
    name: String,
    #[serde(alias = "type")]
    tp: String,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub struct FunHash([u8; 4]);

impl AsRef<[u8; 4]> for FunHash {
    fn as_ref(&self) -> &[u8; 4] {
        &self.0
    }
}

impl From<&AbiEntry> for FunHash {
    fn from(entry: &AbiEntry) -> Self {
        let mut result = [0u8; 4];
        result.copy_from_slice(&Keccak256::digest(&entry.signature())[..4]);
        FunHash(result)
    }
}

impl Debug for FunHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl Display for FunHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
