use crate::bytecode::mir::ir::types::{SType, Value};
use crate::bytecode::mir::translation::Variable;

pub fn bool_const(val: bool) -> Variable {
    Variable::Const(Value::Bool(val), SType::Bool)
}
