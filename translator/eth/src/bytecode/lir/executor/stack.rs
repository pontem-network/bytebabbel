use crate::bytecode::lir::context::Context;
use crate::bytecode::lir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::lir::ir::{Expr, Lir};
use crate::U256;

pub enum StackOp {
    Push(Vec<u8>),
    Pop,
}

impl InstructionHandler for StackOp {
    fn handle(&self, _: Vec<Expr>, _: &mut Lir, _: &mut Context) -> ExecutionResult {
        match self {
            StackOp::Push(val) => ExecutionResult::Output(U256::from(val.as_slice()).into()),
            StackOp::Pop => ExecutionResult::None,
        }
    }
}
