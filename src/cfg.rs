use std::collections::HashMap;
use std::str::FromStr;

use clap::Parser;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, PartialEq)]
// #[clap(author, version, about, long_about = None)]
pub struct Cfg {
    pub address: String,
    pub mapping: HashMap<String, String>,
}

#[derive(Parser, Debug, Clone, PartialEq)]
// #[clap(author, version, about, long_about = None)]
pub struct CfgOverride {
    #[clap(long)]
    pub address: Option<String>,
    /// Format: "0xEthAddr=0xMoveAddr, ..."
    pub mapping: Option<CfgOverrideMapping>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CfgOverrideMapping {
    pub mapping: HashMap<String, String>,
}

impl FromStr for CfgOverrideMapping {
    type Err = clap::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut mapping = HashMap::new();

        s.split(",")
            .into_iter()
            .map(|row| {
                let parts: Vec<_> = row.trim().split("=").map(|s| s.trim()).collect();
                if let (Some(k), Some(v)) = (parts.get(0), parts.get(1)) {
                    mapping.insert(k.to_string(), v.to_string());
                }
            })
            .for_each(|_| {});

        Ok(CfgOverrideMapping { mapping })
    }
}

pub type Abi = Vec<AbiEntry>;

#[derive(Debug, Deserialize)]
pub struct AbiEntry {
    pub name: String,
    #[serde(alias = "type")]
    pub ty: String,
    #[serde(alias = "stateMutability")]
    pub state_mutability: String,
    pub inputs: Vec<AbiTy>,
    pub outputs: Vec<AbiTy>,
}

#[derive(Debug, Deserialize)]
pub struct AbiTy {
    pub name: String,
    #[serde(alias = "type")]
    pub ty: String,
    #[serde(alias = "internalType")]
    pub internal_type: String,
}
