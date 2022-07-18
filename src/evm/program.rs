use crate::evm::abi::{Abi, FunHash};
use crate::evm::bytecode::block::{BlockId, InstructionBlock};
use crate::evm::bytecode::executor::debug::print_flow;
use crate::evm::bytecode::executor::execution::FunctionFlow;
use crate::evm::function::{FunDef, PublicApi};
use anyhow::Error;
use itertools::Itertools;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{Debug, Formatter};

pub struct Program {
    name: String,
    functions_graph: HashMap<FunHash, FunctionFlow>,
    ctor: Option<BTreeMap<BlockId, InstructionBlock>>,
    functions: PublicApi,
}

impl Program {
    pub fn new(
        name: &str,
        functions_graph: HashMap<FunHash, FunctionFlow>,
        ctor: Option<BTreeMap<BlockId, InstructionBlock>>,
        abi: Abi,
    ) -> Result<Program, Error> {
        let functions = PublicApi::new(abi)?;
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

    pub fn public_functions(&self) -> Vec<FunDef> {
        self.functions.function_definition().collect()
    }

    pub fn function_flow(&self, hash: FunHash) -> Option<&FunctionFlow> {
        self.functions_graph.get(&hash)
    }
}

impl Debug for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Program:{}", self.name)?;
        if self.ctor.is_some() {
            writeln!(f, "Ctor detected")?;
        }
        writeln!(f, "Public functions:")?;
        for fun in self.functions.function_definition() {
            write!(f, "fun {} ", fun.abi.signature())?;
            let outputs = fun.abi.outputs();
            if !outputs.is_empty() {
                write!(f, "=> ({})", outputs.iter().map(|o| &o.tp).join(","))?;
            }
            writeln!(f, " {{")?;
            if let Some(flow) = self.functions_graph.get(&fun.hash) {
                print_flow(flow, 5);
            } else {
                writeln!(f, "undefined")?;
            }
            writeln!(f, "}}")?;
        }
        writeln!(f)?;
        Ok(())
    }
}
