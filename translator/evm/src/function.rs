use crate::abi::{Abi, Entry, FunHash};
use anyhow::Error;

pub struct PublicApi {
    abi: Abi,
}

impl PublicApi {
    pub fn new(abi: Abi) -> Result<PublicApi, Error> {
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
                .map(|data| data.inputs.iter().map(|input| input.size()).sum::<usize>())
                .unwrap_or_default()
    }
}
