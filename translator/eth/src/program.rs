use crate::abi::entries::FunHash;
use crate::{Abi, Function, Mir};
use anyhow::Error;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

pub struct Program {
    constructor: Mir,
    functions_mir: HashMap<FunHash, Mir>,
    abi: Abi,
}

impl Program {
    pub fn new(
        constructor: Mir,
        functions_mir: HashMap<FunHash, Mir>,
        abi: Abi,
    ) -> Result<Program, Error> {
        Ok(Program {
            constructor,
            functions_mir,
            abi,
        })
    }

    pub fn name(&self) -> &str {
        self.abi.name()
    }

    pub fn functions_hash(&self) -> impl Iterator<Item = FunHash> + '_ {
        self.abi.functions().iter().map(|f| *f.0)
    }

    pub fn function_def(&self, hash: FunHash) -> Option<&Function> {
        self.abi.functions().get(&hash)
    }

    pub fn function_mir(&self, hash: FunHash) -> Option<&Mir> {
        self.functions_mir.get(&hash)
    }

    pub fn constructor_mir(&self) -> &Mir {
        &self.constructor
    }
}

impl Debug for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Program:{}", self.name())?;
        writeln!(f, "Functions:")?;
        let output = self.debug_constructors();
        writeln!(f, "{output}")?;
        writeln!(f)?;
        for fun in self.abi.functions().values() {
            let output = self.debug_fundef(fun);
            write!(f, "{output}")?;
        }
        writeln!(f)?;
        Ok(())
    }
}

impl Program {
    pub fn debug_constructors(&self) -> String {
        let mut output = String::new();
        output += "public fun constructor(addr)";
        output += " {";
        output += "\n";
        self.constructor.print_to_buffer(&mut output).unwrap();
        output += "\n";
        output += "}";
        output
    }

    pub fn debug_fundef(&self, fun: &Function) -> String {
        let mut output = String::new();
        output += format!("public fun {}", fun).as_str();
        output += " {";
        if let Some(mir) = self.functions_mir.get(&fun.hash) {
            output += "\n";
            mir.print_to_buffer(&mut output).unwrap();
            output += "\n";
        } else {
            output += "\nundefined\n";
        }
        output += "}";

        output
    }
}
