pub mod call;
pub mod entries;
pub mod inc_ret_param;
pub mod types;

use crate::abi::entries::{AbiEntries, Entry, FunHash};
use crate::abi::inc_ret_param::Param;
use crate::bytecode::types::EthType;
use crate::Function;
use anyhow::{Context, Error};
use std::collections::HashMap;

pub struct MoveAbi {
    name: String,
    functions: HashMap<FunHash, Function>,
}

impl MoveAbi {
    pub fn new(name: &str, abi: AbiEntries) -> Result<MoveAbi, Error> {
        let functions = abi
            .entries
            .into_iter()
            .filter_map(|entry| {
                let hash = FunHash::from(&entry);
                if let Entry::Function(fun) = entry {
                    let move_input = vec![EthType::Address, EthType::Bytes];
                    let move_output = vec![EthType::Bytes];
                    let eth_input = map_types(fun.inputs.unwrap_or_default())
                        .context("Input mapping")
                        .unwrap();
                    let eth_output = map_types(fun.outputs.unwrap_or_default())
                        .context("Output mapping")
                        .unwrap();
                    let fun = Function {
                        name: fun.name.unwrap_or_else(|| "anonymous".to_string()),
                        move_input,
                        hash,
                        move_output,
                        eth_input,
                        eth_output,
                    };
                    Some((hash, fun))
                } else {
                    None
                }
            })
            .collect();

        Ok(MoveAbi {
            name: name.to_string(),
            functions,
        })
    }

    pub fn functions(&self) -> &HashMap<FunHash, Function> {
        &self.functions
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

fn map_types(types: Vec<Param>) -> Result<Vec<EthType>, Error> {
    types
        .into_iter()
        .map(|param| EthType::try_from(&param))
        .collect()
}
