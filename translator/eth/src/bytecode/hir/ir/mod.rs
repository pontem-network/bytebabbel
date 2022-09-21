use crate::bytecode::hir::ir::debug::print_ir;
use crate::bytecode::hir::ir::statement::Statement;
use crate::bytecode::hir::ir::var::{Expr, VarId, Vars};
use crate::{BlockId, U256};
use std::mem;

mod debug;
pub mod statement;
pub mod var;

#[derive(Default, Debug)]
pub struct Hir {
    vars: Vars,
    instructions: Vec<Statement>,
    code_copy: Vec<BlockId>,
}

impl Hir {
    pub fn create_var(&mut self, var: Expr) -> VarId {
        let id = self.vars.create(var);
        self.instructions.push(Statement::SetVar(id));
        id
    }

    pub fn mstore(&mut self, addr: VarId, var: VarId) {
        self.instructions.push(Statement::MemStore { addr, var });
    }

    pub fn mstore8(&mut self, addr: VarId, var: VarId) {
        self.instructions.push(Statement::MemStore8 { addr, var });
    }

    pub fn sstore(&mut self, addr: VarId, var: VarId) {
        self.instructions.push(Statement::SStore { addr, var });
    }

    pub fn log(&mut self, offset: VarId, len: VarId, topics: Vec<VarId>) {
        self.instructions.push(Statement::Log {
            offset,
            len,
            topics,
        });
    }

    pub fn push_loop(
        &mut self,
        id: BlockId,
        cnd_block: Vec<Statement>,
        cnd: VarId,
        loop_br: Vec<Statement>,
        is_true_br_loop: bool,
    ) {
        self.instructions.push(Statement::Loop {
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
        true_branch: Vec<Statement>,
        false_branch: Vec<Statement>,
    ) {
        self.instructions.push(Statement::If {
            condition,
            true_branch,
            false_branch,
        });
    }

    pub fn push_continue(&mut self, loop_id: BlockId, context: Vec<Statement>) {
        self.instructions
            .push(Statement::Continue { loop_id, context })
    }

    pub fn print(&self, name: &str) {
        print_ir(self, name);
    }

    pub fn swap_instruction(&mut self, mut instruction: Vec<Statement>) -> Vec<Statement> {
        mem::swap(&mut self.instructions, &mut instruction);
        instruction
    }

    pub fn resolve_var(&self, id: VarId) -> Option<U256> {
        self.vars.resolve_var(id)
    }

    pub fn var(&self, id: &VarId) -> &Expr {
        self.vars.get(id)
    }

    pub fn stop(&mut self) {
        self.instructions.push(Statement::Stop);
    }

    pub fn abort(&mut self, code: u8) {
        self.instructions.push(Statement::Abort(code));
    }

    pub fn code_copy(&mut self, id: BlockId) {
        self.code_copy.push(id);
    }

    pub fn map_var(&mut self, id: VarId, val: VarId) {
        self.instructions.push(Statement::MapVar { id, val });
    }

    pub fn result(&mut self, offset: VarId, len: VarId) {
        self.instructions.push(Statement::Result { offset, len });
    }

    pub fn into_inner(self) -> (Vars, Vec<Statement>, Vec<BlockId>) {
        (self.vars, self.instructions, self.code_copy)
    }

    pub fn get_code_copy(&self) -> &[BlockId] {
        &self.code_copy
    }

    pub fn set_code_copy(&mut self, code_copy: Vec<BlockId>) {
        self.code_copy = code_copy;
    }
}

impl AsRef<[Statement]> for Hir {
    fn as_ref(&self) -> &[Statement] {
        &self.instructions
    }
}
