use std::collections::{HashMap, HashSet};

use anyhow::{Context, Error};
use ethabi::{Contract, Param};

use crate::abi::call::FunHash;
use crate::bytecode::types::EthType;
use crate::Function;

pub mod call;

pub struct MoveAbi {
    name: String,
    identifiers: HashSet<String>,
    functions: HashMap<FunHash, Function>,
}

impl MoveAbi {
    pub fn new(name: &str, abi: &Contract) -> Result<MoveAbi, Error> {
        let (functions, identifiers) = abi
            .functions()
            .map(|fun| {
                let hash = FunHash::from(fun.short_signature());
                (hash, map_function(hash, fun))
            })
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

fn map_function(hash: FunHash, fun: &ethabi::Function) -> Function {
    let move_input = vec![EthType::Address, EthType::Bytes];
    let move_output = vec![EthType::Bytes];

    let eth_input = map_types(fun.inputs.clone())
        .context("Input mapping")
        .unwrap();

    let eth_output = map_types(fun.outputs.clone())
        .context("Output mapping")
        .unwrap();

    Function {
        name: fun.name.clone(),
        eth_input: move_input,
        hash,
        eth_output: move_output,
        native_input: eth_input,
        native_output: eth_output,
    }
}
