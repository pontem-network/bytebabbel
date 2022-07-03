use crate::evm::abi::{Abi, Entry, FunHash};
use crate::evm::bytecode::loc::Loc;
use crate::evm::bytecode::statement::{BasicBlock, BlockId};
use anyhow::{anyhow, Error};
use std::collections::{BTreeMap, HashMap};

pub struct PublicApi {
    abi: Abi,
    public_funcs: HashMap<FunHash, BlockId>,
}

impl PublicApi {
    pub fn new(blocks: &BTreeMap<BlockId, Loc<BasicBlock>>, abi: Abi) -> Result<PublicApi, Error> {
        let functions = abi
            .fun_hashes()
            .map(|h| Self::find_function(h, blocks).map(|id| (h, id)))
            .collect::<Result<_, _>>()?;

        Ok(PublicApi {
            abi,
            public_funcs: functions,
        })
    }

    fn find_function(
        h: FunHash,
        blocks: &BTreeMap<BlockId, Loc<BasicBlock>>,
    ) -> Result<BlockId, Error> {
        blocks
            .iter()
            .filter(|(_, block)| {
                block.statements().iter().any(|s| {
                    if let Some(val) = s.as_push() {
                        h.as_ref() == val
                    } else {
                        false
                    }
                })
            })
            .find_map(|(_, block)| block.last_jump().map(|jmp| jmp.1))
            .and_then(|function_offset| {
                blocks
                    .iter()
                    .find(|(id, _)| function_offset == **id)
                    .map(|(id, _)| *id)
            })
            .ok_or_else(|| anyhow!("couldn't find a function with a signature {h}"))
    }

    pub fn entry_points(&self) -> impl Iterator<Item = BlockId> + '_ {
        self.public_funcs.iter().map(|(_, id)| *id)
    }

    pub fn function_definition(&self) -> Vec<FunctionDefinition> {
        self.abi
            .fun_hashes()
            .map(|h| FunctionDefinition {
                abi: self
                    .abi
                    .entry(&h)
                    .expect("Unreachable state. Expected function abi."),
                hash: h,
                entry_point: *self
                    .public_funcs
                    .get(&h)
                    .expect("Unreachable state. Expected function entry point."),
            })
            .collect()
    }
}

pub struct FunctionDefinition<'a> {
    pub abi: &'a Entry,
    pub hash: FunHash,
    pub entry_point: BlockId,
}
