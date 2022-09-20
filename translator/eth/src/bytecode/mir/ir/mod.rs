use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::SType;
use anyhow::Error;

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
    pub fn add_statement(&mut self, statement: Statement) {
        self.statements.push(statement);
    }

    pub fn extend_statements(&mut self, statements: Vec<Statement>) {
        self.statements.extend(statements);
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

    pub fn print_to_buffer(&self, _buffer: &mut String) -> Result<(), Error> {
        // debug::print_buf(self, buffer, 0)
        todo!()
    }
}
