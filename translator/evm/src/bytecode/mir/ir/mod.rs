use crate::bytecode::mir::ir::statement::Statement;

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
}
