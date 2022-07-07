use crate::evm::abi::{Abi, Entry, FunHash};
use crate::evm::bytecode::executor::block::{BlockId, ExecutedBlock, Jump};
use crate::evm::bytecode::loc::Loc;
use crate::evm::flow_graph::FlowGraph;
use anyhow::{anyhow, Error};
use std::collections::{BTreeMap, HashMap};

pub struct PublicApi {
    abi: Abi,
    public_funcs: HashMap<FunHash, BlockId>,
}

impl PublicApi {
    pub fn new(
        functions_graph: &BTreeMap<FunHash, FlowGraph>,
        abi: Abi,
    ) -> Result<PublicApi, Error> {
        let functions = abi
            .fun_hashes()
            .map(|h| {
                let graph = functions_graph
                    .get(&h)
                    .ok_or_else(|| anyhow!("couldn't find a function with a signature {h}"))?;
                Self::find_function(h, graph.blocks()).map(|id| (h, id))
            })
            .collect::<Result<_, _>>()?;

        Ok(PublicApi {
            abi,
            public_funcs: functions,
        })
    }

    fn find_function(
        h: FunHash,
        blocks: &BTreeMap<BlockId, Loc<ExecutedBlock>>,
    ) -> Result<BlockId, Error> {
        blocks
            .iter()
            .filter(|(_, block)| {
                block.instructions().iter().any(|s| {
                    if let Some(val) = s.1.as_push() {
                        h.as_ref() == val
                    } else {
                        false
                    }
                })
            })
            .find_map(|(_, block)| {
                let exec = block.first_execution()?;
                block.last_jump(exec).map(|jmp| match jmp {
                    Jump::Cnd {
                        true_br,
                        false_br: _,
                    } => true_br,
                    Jump::UnCnd(br) => br,
                })
            })
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

    pub fn function_definition(&self) -> impl Iterator<Item = FunctionDefinition> {
        self.abi.fun_hashes().map(|h| FunctionDefinition {
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
    }
}

pub struct FunctionDefinition<'a> {
    pub abi: &'a Entry,
    pub hash: FunHash,
    pub entry_point: BlockId,
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
