pub mod call;
pub mod entries;
pub mod inc_ret_param;
pub mod types;

use crate::abi::entries::{AbiEntries, Entry, FunHash, FunctionData};
use crate::abi::inc_ret_param::Param;
use crate::bytecode::types::EthType;
use crate::Function;
use anyhow::{Context, Error};
use std::collections::{HashMap, HashSet};

pub struct MoveAbi {
    name: String,
    identifiers: HashSet<String>,
    functions: HashMap<FunHash, Function>,
}

impl MoveAbi {
    pub fn new(name: &str, abi: &AbiEntries) -> Result<MoveAbi, Error> {
        let (functions, identifiers) = abi
            .entries
            .iter()
            .filter_map(|entry| {
                let hash = FunHash::from(entry);
                if let Entry::Function(fun) = entry {
                    Some((hash, fun))
                } else {
                    None
                }
            })
            .map(|(hash, fun)| (hash, map_function(hash, fun)))
            .fold((HashMap::new(), HashSet::new()), |mut acc, (hash, fun)| {
                acc.1.insert(fun.name.to_string());
                acc.0.insert(hash, fun);
                acc
            });

        Ok(MoveAbi {
            name: name.to_string(),
            identifiers,
            functions,
        })
    }

    pub fn functions(&self) -> &HashMap<FunHash, Function> {
        &self.functions
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn identifiers(&self) -> &HashSet<String> {
        &self.identifiers
    }
}

fn map_types(types: Vec<Param>) -> Result<Vec<EthType>, Error> {
    types
        .into_iter()
        .map(|param| EthType::try_from(&param))
        .collect()
}

fn map_function(hash: FunHash, fun: &FunctionData) -> Function {
    let move_input = vec![EthType::Address, EthType::Bytes];
    let move_output = vec![EthType::Bytes];
    let eth_input = map_types(fun.inputs.clone().unwrap_or_default())
        .context("Input mapping")
        .unwrap();
    let eth_output = map_types(fun.outputs.clone().unwrap_or_default())
        .context("Output mapping")
        .unwrap();
    Function {
        name: fun.name.clone().unwrap_or_else(|| "anonymous".to_string()),
        eth_input: move_input,
        hash,
        eth_output: move_output,
        native_input: eth_input,
        native_output: eth_output,
    }
}
