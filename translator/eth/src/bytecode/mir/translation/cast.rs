use crate::bytecode::mir::ir::expression::{Cast, Expression};
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::SType;
use crate::bytecode::mir::translation::Variable;
use crate::MirTranslator;
use anyhow::Error;

impl<'a> MirTranslator<'a> {
    pub fn cast(&mut self, from: Variable, to: SType) -> Result<Variable, Error> {
        if from.s_type() == to {
            return Ok(from);
        }

        let cast = Cast::make(from.s_type(), to)?;
        let var = self.variables.borrow_global(to);
        self.mir
            .add_statement(Statement::CreateVar(var, Expression::Cast(from, cast)));
        Ok(var)
    }
}
