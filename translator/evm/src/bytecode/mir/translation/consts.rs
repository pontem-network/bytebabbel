use crate::bytecode::mir::ir::types::{SType, Value};
use crate::bytecode::mir::translation::Variable;
use std::rc::Rc;

pub fn bool_const(val: bool) -> Rc<Variable> {
    Rc::new(Variable::Const(Value::Bool(val), SType::Bool))
}
