pub mod input;
pub mod output;
pub mod types;

use anyhow::Error;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

#[derive(Debug, Default)]
pub struct Abi {
    entries: HashMap<FunHash, Entry>,
}

impl Abi {
    pub fn fun_hashes(&self) -> impl Iterator<Item = FunHash> + '_ {
        self.entries
            .iter()
            .filter(|(_, abi)| abi.is_function())
            .map(|(h, _)| *h)
    }

    pub fn entry(&self, hash: &FunHash) -> Option<&Entry> {
        self.entries.get(hash)
    }
}

impl TryFrom<&str> for Abi {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let entries: Vec<Entry> = serde_json::from_str(value)?;
        let entries = entries
            .into_iter()
            .map(|e| (FunHash::from(&e), e))
            .collect();
        Ok(Abi { entries })
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Entry {
    pub inputs: Vec<EthType>,
    #[serde(default = "default_name")]
    pub name: String,
    #[serde(default = "default_outputs")]
    pub outputs: Vec<EthType>,
    #[serde(alias = "stateMutability")]
    pub state_mutability: String,
    #[serde(alias = "type")]
    pub tp: String,
}

impl Entry {
    pub fn signature(&self) -> String {
        format!(
            "{}({})",
            self.name,
            self.inputs.iter().map(|i| &i.tp).join(",")
        )
    }

    pub fn is_function(&self) -> bool {
        self.tp == "function"
    }

    pub fn outputs(&self) -> &[EthType] {
        &self.outputs
    }
}

fn default_outputs() -> Vec<EthType> {
    vec![]
}

fn default_name() -> String {
    "".to_string()
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct EthType {
    #[serde(alias = "internalType")]
    pub internal_type: String,
    pub name: String,
    #[serde(alias = "type")]
    pub tp: String,
}

impl EthType {
    pub fn size(&self) -> usize {
        32
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Default)]
pub struct FunHash([u8; 4]);

impl AsRef<[u8; 4]> for FunHash {
    fn as_ref(&self) -> &[u8; 4] {
        &self.0
    }
}

impl From<&Entry> for FunHash {
    fn from(entry: &Entry) -> Self {
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
