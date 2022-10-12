use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir::ir::{Expr};

use crate::{Hir, U256};

pub enum StackOp {
    Push(Vec<u8>),
    Pop,
}

impl InstructionHandler for StackOp {
    fn handle(&self, _: Vec<Expr>, _: &mut Hir, _: &mut Context) -> ExecutionResult {
        match self {
            StackOp::Push(val) => ExecutionResult::Output(U256::from(val.as_slice()).into()),
            StackOp::Pop => ExecutionResult::None,
        }
    }
}
