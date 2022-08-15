use crate::bytecode::mir::ir::math::Operation;
use crate::bytecode::mir::ir::types::{LocalIndex, Value};
use crate::bytecode::mir::translation::Variable;
use std::rc::Rc;

#[derive(Debug)]
pub enum Statement {
    CastBool(Rc<Variable>),
    CastUInt(Rc<Variable>),
    Const(Value),
    Not(Rc<Variable>),
    CreateVar(LocalIndex, Box<Statement>),
    MemStore(Rc<Variable>, Rc<Variable>),
    Operation(Operation, Rc<Variable>, Rc<Variable>),
    IF {
        cnd: Rc<Variable>,
        true_br: Vec<Statement>,
        false_br: Vec<Statement>,
    },
}
