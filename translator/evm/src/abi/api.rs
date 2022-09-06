use crate::abi::inc_ret_param::Param;
use crate::abi::types::StateMutability;
use crate::abi::{Abi, Entry, FunHash, FunctionData};
use anyhow::Error;
use std::collections::HashMap;

pub struct PublicApi {
    constructor: Constructor,
    functions: Vec<Function>,
}

impl PublicApi {
    pub fn new(abi: Abi) -> Result<PublicApi, Error> {
        // abi.constructor().clone().map(|f| FunctionData {
        //     name: f.1.name().unwrap_or("constructor".to_string()),
        //     inputs: f.1.inputs(),
        //     outputs: None,
        //     state_mutability: Default::default()
        // }).unwrap_or_else(||)

        Ok(PublicApi { abi })
    }

    pub fn function_definition(&self) -> impl Iterator<Item = FunDef> {
        self.abi.fun_hashes().map(|h| FunDef {
            abi: self
                .abi
                .entry(&h)
                .expect("Unreachable state. Expected function abi."),
            hash: h,
        })
    }

    pub fn get_constructor(&self) -> FunDef {
        if let Some((hash, entry)) = self.abi.constructor() {
            FunDef {
                abi: entry.clone(),
                hash: *hash,
            }
        } else {
            let entry = Entry::Constructor(FunctionData {
                name: Some("constructor".to_string()),
                inputs: Some(vec![]),
                outputs: Some(vec![]),
                state_mutability: StateMutability::Nonpayable,
            });
            FunDef {
                abi: entry,
                hash: *hash,
            }
        }
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

pub struct Constructor {
    inputs: Vec<Param>,
}

pub struct Function {
    pub name: String,
    pub hash: FunHash,
    pub inputs: Vec<Param>,
    pub outputs: Vec<Param>,
    pub state_mutability: StateMutability,
}
