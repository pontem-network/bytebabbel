use crate::bytecode::llir::context::Context;
use crate::bytecode::llir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::llir::stack::StackFrame;
use crate::Ir;
use std::fmt::{Display, Formatter};

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum BinaryOp {
    EQ,
    Lt,
    Gt,
    Shr,
    Shl,
    Sar,
    Add,
    And,
    Or,
    Xor,
    Mul,
    Sub,
    Div,
    SDiv,
    SLt,
    SGt,
    Byte,
    Mod,
    SMod,
    AddMod,
    MulMod,
    Exp,
    SignExtend,
}

impl InstructionHandler for BinaryOp {
    fn handle(
        &self,
        params: Vec<StackFrame>,
        ir: &mut Ir,
        context: &mut Context,
    ) -> ExecutionResult {
        todo!()
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum UnaryOp {
    IsZero,
    Not,
}

impl InstructionHandler for UnaryOp {
    fn handle(
        &self,
        params: Vec<StackFrame>,
        ir: &mut Ir,
        context: &mut Context,
    ) -> ExecutionResult {
        todo!()
    }
}
