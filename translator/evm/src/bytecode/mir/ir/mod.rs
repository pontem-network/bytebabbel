use crate::bytecode::mir::ir::statement::Statement;
use std::mem;

pub mod debug;
pub mod expression;
pub mod math;
pub mod statement;
pub mod types;

#[derive(Debug, Default)]
pub struct Mir {
    statements: Vec<Statement>,
}

impl Mir {
    pub fn add_statement(&mut self, statement: Statement) {
        println!("{:?}", statement);
        self.statements.push(statement);
    }

    pub fn swap(&mut self, mut mir: Mir) -> Mir {
        mem::swap(&mut self.statements, &mut mir.statements);
        mir
    }

    pub fn into_inner(self) -> Vec<Statement> {
        self.statements
    }

    pub fn as_statements(&self) -> &[Statement] {
        &self.statements
    }

    pub fn print(&self) {
        debug::print_ir(self);
    }
}
