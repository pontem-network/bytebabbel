use crate::bytecode::hir::ir::debug::print_ir;
use crate::bytecode::hir::ir::instruction::Instruction;
use crate::bytecode::hir::ir::var::{Var, VarId, Vars};
use crate::{BlockId, U256};
use std::mem;

mod debug;
pub mod instruction;
pub mod var;

#[derive(Default, Debug)]
pub struct Hir {
    vars: Vars,
    instructions: Vec<Instruction>,
}

impl Hir {
    pub fn create_var(&mut self, var: Var) -> VarId {
        let id = self.vars.create(var);
        self.instructions.push(Instruction::SetVar(id));
        id
    }

    pub fn mstore(&mut self, addr: VarId, var: VarId) {
        self.instructions.push(Instruction::MemStore { addr, var });
    }

    pub fn mstore8(&mut self, addr: VarId, var: VarId) {
        self.instructions.push(Instruction::MemStore8 { addr, var });
    }

    pub fn sstore(&mut self, addr: VarId, var: VarId) {
        self.instructions.push(Instruction::SStore { addr, var });
    }

    pub fn push_loop(
        &mut self,
        id: BlockId,
        cnd_block: Vec<Instruction>,
        cnd: VarId,
        loop_br: Vec<Instruction>,
        is_true_br_loop: bool,
    ) {
        self.instructions.push(Instruction::Loop {
            id,
            condition_block: cnd_block,
            condition: cnd,
            is_true_br_loop,
            loop_br,
        });
    }

    pub fn push_if(
        &mut self,
        condition: VarId,
        true_branch: Vec<Instruction>,
        false_branch: Vec<Instruction>,
    ) {
        self.instructions.push(Instruction::If {
            condition,
            true_branch,
            false_branch,
        });
    }

    pub fn push_continue(&mut self, loop_id: BlockId, context: Vec<Instruction>) {
        self.instructions
            .push(Instruction::Continue { loop_id, context })
    }

    pub fn print(&self, name: &str) {
        print_ir(self, name);
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

    pub fn stop(&mut self) {
        self.instructions.push(Instruction::Stop);
    }

    pub fn abort(&mut self, code: u8) {
        self.instructions.push(Instruction::Abort(code));
    }

    pub fn code_copy(&mut self, id: BlockId) {
        self.instructions.push(Instruction::CodeCopy(id));
    }

    pub fn map_var(&mut self, id: VarId, val: VarId) {
        self.instructions.push(Instruction::MapVar { id, val });
    }

    pub fn result(&mut self, offset: VarId, len: VarId) {
        self.instructions.push(Instruction::Result { offset, len });
    }

    pub fn into_inner(self) -> (Vars, Vec<Instruction>) {
        (self.vars, self.instructions)
    }
}

impl AsRef<[Instruction]> for Hir {
    fn as_ref(&self) -> &[Instruction] {
        &self.instructions
    }
}
