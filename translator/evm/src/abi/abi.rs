use crate::abi::entries::{AbiEntries, Entry, FunHash};
use crate::abi::inc_ret_param::Param;
use crate::bytecode::types::{Constructor, EthType};
use crate::Function;
use anyhow::{Context, Error};
use std::collections::HashMap;

pub struct Abi {
    name: String,
    constructor: Constructor,
    functions: HashMap<FunHash, Function>,
}

impl Abi {
    pub fn new(name: &str, abi: AbiEntries) -> Result<Abi, Error> {
        let (constructor, functions) = abi.entries.into_iter().fold(
            (None, HashMap::new()),
            |(mut constructor, mut functions), entry| {
                let hash = FunHash::from(&entry);
                match entry {
                    Entry::Function(fun) => {
                        functions.insert(
                            hash,
                            Function {
                                name: fun.name.unwrap_or_else(|| "anonymous".to_string()),
                                input: map_types(fun.inputs.unwrap_or_default())
                                    .context("Input mapping")
                                    .unwrap(),
                                hash,
                                output: map_types(fun.outputs.unwrap_or_default())
                                    .context("Output mapping")
                                    .unwrap(),
                            },
                        );
                    }
                    Entry::Constructor(fun) => {
                        if constructor.is_some() {
                            panic!("Multiple constructors are not supported");
                        }
                        let mut new_constructor = Constructor::default();
                        new_constructor
                            .input
                            .extend(map_types(fun.inputs.unwrap_or_default()).unwrap());
                        constructor = Some(new_constructor);
                    }
                    _ => {
                        todo!("unimplemented entry")
                    }
                }
                (constructor, functions)
            },
        );

        Ok(Abi {
            name: name.to_string(),
            constructor: constructor.unwrap_or_default(),
            functions,
        })
    }

    pub fn constructor(&self) -> &Constructor {
        &self.constructor
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
