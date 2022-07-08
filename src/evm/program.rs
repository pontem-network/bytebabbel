use crate::evm::abi::{Abi, FunHash};
use crate::evm::bytecode::block::InstructionBlock;
use crate::evm::bytecode::executor::block::BlockId;
use crate::evm::flow_graph::FlowGraph;
use crate::evm::function::{FunctionDefinition, PublicApi};
use anyhow::Error;
use itertools::Itertools;
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};

pub struct Program {
    name: String,
    functions_graph: BTreeMap<FunHash, FlowGraph>,
    ctor: Option<BTreeMap<BlockId, InstructionBlock>>,
    functions: PublicApi,
}

impl Program {
    pub fn new(
        name: &str,
        functions_graph: BTreeMap<FunHash, FlowGraph>,
        ctor: Option<BTreeMap<BlockId, InstructionBlock>>,
        abi: Abi,
    ) -> Result<Program, Error> {
        let functions = PublicApi::new(&functions_graph, abi)?;
        Ok(Program {
            name: name.to_string(),
            functions_graph,
            ctor,
            functions,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn public_functions(&self) -> Vec<FunctionDefinition> {
        self.functions.function_definition().collect()
    }

    pub fn flow_graph(&self, hash: FunHash) -> Option<&FlowGraph> {
        self.functions_graph.get(&hash)
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
        Ok(())
    }
}
