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

#[derive(Hash, Eq, PartialEq, Copy, Clone, Default, Debug, Ord, PartialOrd)]
pub struct VarId(u64);

impl VarId {
    pub fn index(&self) -> u64 {
        self.0
    }
}
