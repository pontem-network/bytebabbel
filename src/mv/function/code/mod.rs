use crate::evm::bytecode::executor::block::{BlockId, Chain};
use crate::evm::function::FunctionDefinition;
use crate::evm::program::Program;
use crate::mv::function::code::instruction::InstractionTree;
use anyhow::{anyhow, Error};
use move_binary_format::file_format::CodeUnit;

pub mod instruction;

#[derive(Debug)]
pub struct MvIr {}

impl MvIr {
    pub fn make_ir(def: FunctionDefinition, program: &Program) -> Result<MvIr, Error> {
        let graph = program
            .flow_graph(def.hash)
            .ok_or_else(|| anyhow!("Root path for {} function not found.", def.abi.name))?;

        let mut ir = Vec::new();
        let mut chain = Chain::default();
        let entry_block = graph
            .block(&BlockId::from(0))
            .ok_or_else(|| anyhow!("Function body {} not found.", def.abi.name))?;

        let exec = entry_block
            .executions
            .get(&chain)
            .ok_or_else(|| anyhow!("Execution of {}:{:?} not found.", def.abi.name, chain))?;

        for item in &exec.out_stack_items {
            if !item.is_used() {
                ir.push(InstractionTree::from(item)?);
            }
        }
        Ok(MvIr {})
    }

    pub fn move_byte_code(&self) -> Result<CodeUnit, Error> {
        Ok(CodeUnit {
            locals: Default::default(),
            code: vec![],
        })
    }
}

#[derive(Debug)]
pub enum ExecutionTree {}
