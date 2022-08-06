use crate::bytecode::llir::ir::debug::print_ir;
use crate::bytecode::llir::ir::instruction::Instruction;
use crate::bytecode::llir::ir::var::{Var, VarId, Vars};
use crate::U256;
use std::mem;

mod debug;
mod instruction;
pub mod var;

#[derive(Default, Debug)]
pub struct Ir {
    vars: Vars,
    instructions: Vec<Instruction>,
}

impl Ir {
    pub fn create_var(&mut self, var: Var) -> VarId {
        let id = self.vars.create(var);
        self.instructions.push(Instruction::SetVar(id));
        id
    }

    pub fn mem_store(&mut self, offset: U256, val: VarId) {
        self.instructions.push(Instruction::MemStore(offset, val));
    }

    pub fn push_if(
        &mut self,
        condition: VarId,
        true_branch: Vec<Instruction>,
        false_branch: Vec<Instruction>,
    ) {
        self.instructions.push(Instruction::Branch {
            condition,
            true_branch_len: true_branch.len() as u64,
            false_branch_len: false_branch.len() as u64,
        });
        self.instructions.extend(true_branch);
        self.instructions.extend(false_branch);
    }

    pub fn print(&self) {
        print_ir(self);
    }

    pub fn swap_instruction(&mut self, mut instruction: Vec<Instruction>) -> Vec<Instruction> {
        mem::swap(&mut self.instructions, &mut instruction);
        instruction
    }

    pub fn resolve_var(&self, id: VarId) -> Option<U256> {
        self.vars.resolve_var(id)
    }

    pub fn var(&self, id: &VarId) -> &Var {
        self.vars.get(id)
    }
}
