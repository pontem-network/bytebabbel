use crate::bytecode::hir::ir::statement::Statement;

pub mod debug;
pub mod expression;
pub mod statement;

#[derive(Debug, Default, Clone)]
pub struct Hir {
    statements: Vec<Statement>,
}

impl Hir {
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

impl From<Vec<Statement>> for Hir {
    fn from(statements: Vec<Statement>) -> Self {
        Hir { statements }
    }
}
