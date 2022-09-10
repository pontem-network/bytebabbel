pub mod call;
pub mod entries;
pub mod inc_ret_param;
pub mod types;

use crate::abi::entries::{AbiEntries, Entry, FunHash};
use crate::bytecode::types::EthType;
use crate::Function;
use anyhow::Error;
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
                    let input = vec![EthType::Address, EthType::Bytes];
                    let output = vec![EthType::Bytes];
                    // todo replace with static type definition
                    // input.extend(
                    //     map_types(fun.inputs.unwrap_or_default())
                    //         .context("Input mapping")
                    //         .unwrap(),
                    // );
                    // let output = map_types(fun.outputs.unwrap_or_default())
                    //     .context("Output mapping")
                    //     .unwrap();
                    let fun = Function {
                        name: fun.name.unwrap_or_else(|| "anonymous".to_string()),
                        input,
                        hash,
                        output,
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
