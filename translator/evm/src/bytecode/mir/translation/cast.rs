use crate::bytecode::mir::ir::expression::Expression;
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::{SType, Value};
use crate::bytecode::mir::translation::Variable;
use crate::MirTranslator;
use anyhow::Error;

impl MirTranslator {
    pub fn cast_number(&mut self, var: Variable) -> Result<Variable, Error> {
        match var.s_type() {
            SType::U128 => Ok(var),
            SType::Bool => {
                let result = self.variables.borrow(SType::U128);
                self.mir.add_statement(Statement::IF {
                    cnd: var.expr(),
                    true_br: vec![Statement::CreateVar(
                        result,
                        Expression::Const(Value::U128(1)),
                    )],
                    false_br: vec![Statement::CreateVar(
                        result,
                        Expression::Const(Value::U128(0)),
                    )],
                });
                self.variables.release(result);
                Ok(Variable(result, SType::U128))
            }
        }
    }
}
