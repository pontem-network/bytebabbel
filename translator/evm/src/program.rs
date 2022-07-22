use crate::abi::{Abi, FunHash};
use crate::bytecode::block::{BlockId, InstructionBlock};
use crate::bytecode::executor::debug::output_flow;
use crate::bytecode::executor::execution::FunctionFlow;
use crate::function::{FunDef, PublicApi};
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
        writeln!(f)?;
        for fun in self.functions.function_definition() {
            let output = self.debug_fundef(&fun);
            write!(f, "{output}")?;
        }
        writeln!(f)?;
        Ok(())
    }
}

impl Program {
    pub fn debug_fundef(&self, fundef: &FunDef) -> String {
        let mut output = String::new();
        output += format!("public fun {} ", fundef.abi.signature()).as_str();
        let outputs = fundef.abi.outputs();
        if !outputs.is_empty() {
            output += format!("=> ({})", outputs.iter().map(|o| &o.tp).join(",")).as_str();
        }
        output += " {";
        if let Some(flow) = self.functions_graph.get(&fundef.hash) {
            output += format!("\n{}\n", output_flow(flow, 5)).as_str();
        } else {
            output += "\nundefined\n";
        }
        output += "}";

        output
    }
    pub fn debug_fn_by_hash(&self, hash: FunHash) -> String {
        self.functions
            .function_definition()
            .find(|item| item.hash == hash)
            .as_ref()
            .map(|fun| self.debug_fundef(fun))
            .unwrap_or_default()
    }
}
