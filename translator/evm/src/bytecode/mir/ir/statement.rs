use crate::bytecode::mir::ir::math::Operation;
use crate::bytecode::mir::ir::types::LocalIndex;
use crate::bytecode::mir::translation::Variable;
use std::rc::Rc;

#[derive(Debug)]
pub enum Statement {
    CastBool(Rc<Variable>),
    CastUInt(Rc<Variable>),
    Not(Rc<Variable>),
    CreateVar(LocalIndex, Box<Statement>),
    Operation(Operation, Rc<Variable>, Rc<Variable>),
}
