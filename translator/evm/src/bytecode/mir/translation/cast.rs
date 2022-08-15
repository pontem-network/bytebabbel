use crate::bytecode::mir::ir::statement::{Statement, VarOrStack};
use crate::bytecode::mir::ir::types::{SType, Value};
use crate::bytecode::mir::translation::Variable;
use crate::MirTranslator;
use anyhow::Error;
use std::rc::Rc;

impl MirTranslator {
    pub fn cast_number(&mut self, var: Variable) -> Result<Variable, Error> {
        match var.s_type() {
            SType::U128 => Ok(var),
            SType::Bool => {
                let result = self.variables.borrow_local(SType::U128);
                self.mir.add_statement(Statement::IF {
                    cnd: VarOrStack::Var(var),
                    true_br: vec![Statement::CreateVar(
                        result,
                        Box::new(Statement::Const(Value::U128(1))),
                    )],
                    false_br: vec![Statement::CreateVar(
                        result,
                        Box::new(Statement::Const(Value::U128(0))),
                    )],
                });
                self.variables.release_local(result);
                Ok(Variable::LocalBorrow(result, SType::U128))
            }
        }
    }
}
