use clap::Parser;
use serde::Deserialize;

#[derive(Parser, Deserialize, Debug, Clone, PartialEq)]
// #[clap(author, version, about, long_about = None)]
pub struct Cfg {
    #[clap(long)]
    pub address: String,
}

#[derive(Parser, Debug, Clone, PartialEq)]
// #[clap(author, version, about, long_about = None)]
pub struct CfgOverride {
    #[clap(long)]
    pub address: Option<String>,
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
