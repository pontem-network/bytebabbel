use crate::bytecode::mir::ir::expression::Expression;
use crate::bytecode::mir::ir::types::Value;

pub fn bool_const(val: bool) -> Expression {
    Expression::Const(Value::from(val))
}
