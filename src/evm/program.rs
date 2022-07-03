use crate::evm::abi::Abi;
use crate::evm::bytecode::flow_graph::ControlFlowGraph;
use crate::evm::bytecode::loc::Loc;
use crate::evm::bytecode::statement::{BasicBlock, BlockId};
use crate::evm::function::Functions;
use anyhow::Error;
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};

pub struct Program {
    name: String,
    blocks: BTreeMap<BlockId, Loc<BasicBlock>>,
    ctor: Option<BTreeMap<BlockId, Loc<BasicBlock>>>,
    flow_graph: ControlFlowGraph,
    functions: Functions,
}

impl Program {
    pub fn new(
        name: &str,
        blocks: BTreeMap<BlockId, Loc<BasicBlock>>,
        ctor: Option<BTreeMap<BlockId, Loc<BasicBlock>>>,
        abi: Abi,
    ) -> Result<Program, Error> {
        let functions = Functions::new(&blocks, abi)?;
        let flow_graph = ControlFlowGraph::new(&blocks, functions.entry_points())?;

        Ok(Program {
            name: name.to_string(),
            blocks,
            ctor,
            flow_graph,
            functions,
        })
    }
}

impl Debug for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "entry_points: {:?}", self.flow_graph.entry_points())?;
        writeln!(f, "blocks: {:?}", self.flow_graph.blocks())
    }
}
