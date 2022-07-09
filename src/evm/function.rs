use crate::evm::abi::{Abi, Entry, FunHash};
use anyhow::Error;

pub struct PublicApi {
    abi: Abi,
}

impl PublicApi {
    pub fn new(abi: Abi) -> Result<PublicApi, Error> {
        Ok(PublicApi { abi })
    }

    pub fn function_definition(&self) -> impl Iterator<Item = FunctionDefinition> {
        self.abi.fun_hashes().map(|h| FunctionDefinition {
            abi: self
                .abi
                .entry(&h)
                .expect("Unreachable state. Expected function abi."),
            hash: h,
        })
    }
}

pub struct FunctionDefinition<'a> {
    pub abi: &'a Entry,
    pub hash: FunHash,
}

impl<'a> FunctionDefinition<'a> {
    pub fn input_size(&self) -> usize {
        self.hash.as_ref().len()
            + self
                .abi
                .inputs
                .iter()
                .map(|input| input.size())
                .sum::<usize>()
    }
}
