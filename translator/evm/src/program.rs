use crate::abi::{Abi, FunHash};
use crate::bytecode::block::{BlockId, InstructionBlock};
use crate::function::{FunDef, PublicApi};
use crate::Mir;
use anyhow::Error;
use itertools::Itertools;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

pub struct Program {
    name: String,
    functions_mir: HashMap<FunHash, Mir>,
    ctor: Option<HashMap<BlockId, InstructionBlock>>,
    functions: PublicApi,
}

impl Program {
    pub fn new(
        name: &str,
        functions_mir: HashMap<FunHash, Mir>,
        ctor: Option<HashMap<BlockId, InstructionBlock>>,
        abi: Abi,
    ) -> Result<Program, Error> {
        let functions = PublicApi::new(abi)?;
        Ok(Program {
            name: name.to_string(),
            functions_mir,
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

    pub fn function_mir(&self, hash: FunHash) -> Option<&Mir> {
        self.functions_mir.get(&hash)
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
        if let Some(outputs) = fundef.abi.function_data().and_then(|data| data.outputs()) {
            if !outputs.is_empty() {
                output += format!(
                    "=> ({})",
                    outputs.iter().map(|o| o.tp.to_string()).join(",")
                )
                .as_str();
            }
        }
        output += " {";
        if let Some(mir) = self.functions_mir.get(&fundef.hash) {
            output += "\n";
            mir.print_to_buffer(&mut output).unwrap();
            output += "\n";
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
