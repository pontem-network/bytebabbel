use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir2::ir::{Expr, Hir2};
use crate::U256;

pub enum StackOp {
    Push(Vec<u8>),
    Pop,
}

impl InstructionHandler for StackOp {
    fn handle(&self, _: Vec<Expr>, _: &mut Hir2, _: &mut Context) -> ExecutionResult {
        match self {
            StackOp::Push(val) => ExecutionResult::Output(U256::from(val.as_slice()).into()),
            StackOp::Pop => ExecutionResult::None,
        }
    }
}
