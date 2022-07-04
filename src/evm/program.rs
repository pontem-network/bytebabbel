use crate::evm::abi::Abi;
use crate::evm::bytecode::flow_graph::ControlFlowGraph;
use crate::evm::bytecode::loc::Loc;
use crate::evm::bytecode::statement::{BasicBlock, BlockId};
use crate::evm::function::{FunctionDefinition, PublicApi};
use anyhow::Error;
use itertools::Itertools;
use std::collections::{BTreeMap, HashSet};
use std::fmt::{Debug, Formatter};

pub struct Program {
    name: String,
    blocks: BTreeMap<BlockId, Loc<BasicBlock>>,
    ctor: Option<BTreeMap<BlockId, Loc<BasicBlock>>>,
    flow_graph: ControlFlowGraph,
    functions: PublicApi,
    private_functions: HashSet<BlockId>,
}

impl Program {
    pub fn new(
        name: &str,
        blocks: BTreeMap<BlockId, Loc<BasicBlock>>,
        ctor: Option<BTreeMap<BlockId, Loc<BasicBlock>>>,
        abi: Abi,
    ) -> Result<Program, Error> {
        let functions = PublicApi::new(&blocks, abi)?;
        let entry_points = functions
            .function_definition()
            .map(|def| (def.entry_point, def.input_size()));
        let flow_graph = ControlFlowGraph::new(&blocks, entry_points)?;

        let flow = flow_graph.build_flow(*flow_graph.entry_points().iter().next().unwrap())?;
        dbg!(flow);

        Ok(Program {
            name: name.to_string(),
            blocks,
            ctor,
            flow_graph,
            functions,
            private_functions: Default::default(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn public_functions(&self) -> Vec<FunctionDefinition> {
        self.functions.function_definition().collect()
    }
}

impl Debug for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Program:{}", self.name)?;
        writeln!(f, "Public functions:")?;
        for fun in self.functions.function_definition() {
            write!(f, "fun {} ", fun.abi.signature())?;
            let outputs = fun.abi.outputs();
            if !outputs.is_empty() {
                write!(f, "=> ({})", outputs.iter().map(|o| &o.tp).join(","))?;
            }
            writeln!(f, " {{")?;
            // todo function instructions.
            writeln!(f, "   Block code:{}", fun.entry_point)?;
            writeln!(f, "}}")?;
        }
        writeln!(f)?;
        writeln!(f, "Private functions:")?;
        for id in &self.private_functions {
            write!(f, "fun {} ", hex::encode(id.to_le_bytes()))?;
        }
        Ok(())
    }
}
