use crate::evm::abi::Abi;
use crate::evm::bytecode::block::InstructionBlock;
use crate::evm::bytecode::executor::{BlockId, ExecutedBlock};
use crate::evm::bytecode::loc::Loc;
use crate::evm::function::{FunctionDefinition, PublicApi};
use anyhow::Error;
use itertools::Itertools;
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};

pub struct Program {
    name: String,
    blocks: BTreeMap<BlockId, Loc<ExecutedBlock>>,
    ctor: Option<BTreeMap<BlockId, InstructionBlock>>,
    functions: PublicApi,
}

impl Program {
    pub fn new(
        name: &str,
        blocks: BTreeMap<BlockId, Loc<ExecutedBlock>>,
        ctor: Option<BTreeMap<BlockId, InstructionBlock>>,
        abi: Abi,
    ) -> Result<Program, Error> {
        let functions = PublicApi::new(&blocks, abi)?;
        Ok(Program {
            name: name.to_string(),
            blocks,
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
