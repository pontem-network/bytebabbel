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

pub struct Abi {
    name: String,
    functions: HashMap<FunHash, Function>,
}

impl Abi {
    pub fn new(name: &str, abi: AbiEntries) -> Result<Abi, Error> {
        let functions = abi
            .entries
            .into_iter()
            .filter_map(|entry| {
                let hash = FunHash::from(&entry);
                if let Entry::Function(fun) = entry {
                    let fun = Function {
                        name: fun.name.unwrap_or_else(|| "anonymous".to_string()),
                        input: map_types(fun.inputs.unwrap_or_default())
                            .context("Input mapping")
                            .unwrap(),
                        hash,
                        output: map_types(fun.outputs.unwrap_or_default())
                            .context("Output mapping")
                            .unwrap(),
                    };
                    Some((hash, fun))
                } else {
                    None
                }
            })
            .collect();

        Ok(Abi {
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
