use crate::bytecode::hir2::ir::statement::Statement;

pub mod debug;
pub mod expression;
pub mod statement;

#[derive(Debug, Default, Clone)]
pub struct Hir2 {
    statements: Vec<Statement>,
}

impl Hir2 {
    pub fn add_statement(&mut self, statement: Statement) {
        self.statements.push(statement);
    }

    pub fn statements(&self) -> &[Statement] {
        &self.statements
    }

    pub fn inner(self) -> Vec<Statement> {
        self.statements
    }
}
