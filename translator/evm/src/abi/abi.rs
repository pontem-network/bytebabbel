use crate::abi::entries::{AbiEntries, Entry, FunHash};
use crate::abi::inc_ret_param::Param;
use crate::abi::types::StateMutability;
use anyhow::Error;

pub struct Abi {
    name: String,
    constructor: Constructor,
    functions: Vec<Function>,
}

impl Abi {
    pub fn new(name: &str, abi: AbiEntries) -> Result<Abi, Error> {
        let (constructor, functions) = abi.entries.into_iter().fold(
            (None, Vec::new()),
            |(mut constructor, mut functions), entry| {
                let hash = FunHash::from(&entry);
                match entry {
                    Entry::Function(fun) => {
                        functions.push(Function {
                            name: fun.name.unwrap_or_else(|| "anonymous".to_string()),
                            hash,
                            inputs: fun.inputs.unwrap_or_default(),
                            outputs: fun.outputs.unwrap_or_default(),
                            state_mutability: fun.state_mutability,
                        });
                    }
                    Entry::Constructor(fun) => {
                        if constructor.is_some() {
                            panic!("Multiple constructors are not supported");
                        }
                        constructor = Some(Constructor {
                            inputs: fun.inputs.unwrap_or_default(),
                        });
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
}

pub struct FunDef<'a> {
    pub abi: &'a Entry,
    pub hash: FunHash,
}

impl<'a> FunDef<'a> {
    pub fn input_size(&self) -> usize {
        self.hash.as_ref().len()
            + self
                .abi
                .function_data()
                .map(|data| {
                    data.inputs
                        .as_ref()
                        .map(|inp| inp.iter().map(|input| input.size()).sum::<usize>())
                        .unwrap_or_default()
                })
                .unwrap_or_default()
    }
}

#[derive(Default)]
pub struct Constructor {
    pub inputs: Vec<Param>,
}

pub struct Function {
    pub name: String,
    pub hash: FunHash,
    pub inputs: Vec<Param>,
    pub outputs: Vec<Param>,
    pub state_mutability: StateMutability,
}
