use crate::abi::api::{FunDef, PublicApi};
use crate::abi::{Abi, FunHash};
use crate::bytecode::block::{BlockId, InstructionBlock};
use crate::Mir;
use anyhow::Error;
use itertools::Itertools;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

pub struct Program {
    name: String,
    constructor: Mir,
    functions_mir: HashMap<FunHash, Mir>,
    api: PublicApi,
}

impl Program {
    pub fn new(
        name: &str,
        constructor: Mir,
        functions_mir: HashMap<FunHash, Mir>,
        abi: Abi,
    ) -> Result<Program, Error> {
        let functions = PublicApi::new(abi)?;
        Ok(Program {
            name: name.to_string(),
            constructor,
            functions_mir,
            api: functions,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn public_functions(&self) -> Vec<FunDef> {
        self.api.function_definition().collect()
    }

    pub fn function_mir(&self, hash: FunHash) -> Option<&Mir> {
        self.functions_mir.get(&hash)
    }
}

impl Debug for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Program:{}", self.name)?;
        writeln!(f, "Functions:")?;
        let output = self.debug_constructors()?;
        writeln!(f, "{output}")?;
        writeln!(f)?;
        for fun in self.api.function_definition() {
            let output = self.debug_fundef(&fun);
            write!(f, "{output}")?;
        }
        writeln!(f)?;
        Ok(())
    }
}

impl Program {
    pub fn debug_constructors(&self) -> String {
        let mut output = String::new();

        output
    }

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
        self.api
            .function_definition()
            .find(|item| item.hash == hash)
            .as_ref()
            .map(|fun| self.debug_fundef(fun))
            .unwrap_or_default()
    }
}
