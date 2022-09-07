use crate::bytecode::mir::ir::expression::{Expression, StackOpsBuilder};
use crate::bytecode::mir::ir::math::Operation;
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::{SType, Value};
use crate::bytecode::mir::translation::Variable;
use crate::MirTranslator;
use anyhow::{anyhow, Error};

impl<'a> MirTranslator<'a> {
    pub fn cast_number(&mut self, var: Variable) -> Result<Variable, Error> {
        match var.s_type() {
            SType::Number => Ok(var),
            SType::Bool => {
                let result = self.variables.borrow(SType::Number);
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
                Ok(result)
            }
            SType::Storage => Err(anyhow!("Storage type not supported for cast")),
            SType::Memory => Err(anyhow!("Memory type not supported for cast")),
            SType::Address => {
                todo!("Cast address to number")
            }
        }
    }

    pub fn cast_bool(&mut self, var: Variable) -> Result<Variable, Error> {
        match var.s_type() {
            SType::Bool => Ok(var),
            SType::Number => {
                let result = self.variables.borrow(SType::Bool);
                let cnd = StackOpsBuilder::default()
                    .push_const(Value::U128(0))
                    .push_var(var)
                    .binary_op(Operation::Neq, SType::Number, SType::Bool)?
                    .build(SType::Bool)?;

                self.mir.add_statement(Statement::IF {
                    cnd,
                    true_br: vec![Statement::CreateVar(
                        result,
                        Expression::Const(Value::Bool(true)),
                    )],
                    false_br: vec![Statement::CreateVar(
                        result,
                        Expression::Const(Value::Bool(false)),
                    )],
                });
                Ok(result)
            }
            SType::Storage => Err(anyhow!("Storage type not supported for cast")),
            SType::Memory => Err(anyhow!("Memory type not supported for cast")),
            SType::Address => {
                todo!()
            }
        }
    }
}
