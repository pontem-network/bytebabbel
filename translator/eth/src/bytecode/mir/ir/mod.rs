use std::mem;

use anyhow::Error;

use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::SType;

pub mod debug;
pub mod expression;
pub mod statement;
pub mod types;

#[derive(Debug, Default, Clone)]
pub struct Mir {
    locals: Vec<SType>,
    statements: Vec<Statement>,
}

impl Mir {
    pub fn push(&mut self, statement: Statement) {
        self.statements.push(statement);
    }

    pub fn swap(&mut self, mut mir: Mir) -> Mir {
        mem::swap(&mut self.statements, &mut mir.statements);
        mir
    }

    pub fn into_inner(self) -> Vec<Statement> {
        self.statements
    }

    pub fn statements(&self) -> &[Statement] {
        &self.statements
    }

    pub fn locals(&self) -> &[SType] {
        &self.locals
    }

    pub fn set_locals(&mut self, locals: Vec<SType>) {
        self.locals = locals;
    }

    pub fn print(&self, name: &str) {
        debug::print_ir(self, name);
    }

    pub fn print_to_buffer(&self, buffer: &mut String) -> Result<(), Error> {
        debug::print_buf(self, buffer, 0)
    }
}
