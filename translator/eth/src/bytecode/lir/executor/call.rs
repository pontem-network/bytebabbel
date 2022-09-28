use crate::bytecode::hir::context::Context;
use crate::bytecode::lir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::lir::ir::{Expr, Lir};
use crate::Hir;

pub enum CallOp {
    Call,
    CallCode,
    DelegateCall,
    StaticCall,
}

impl InstructionHandler for CallOp {
    fn handle(&self, _: Vec<Expr>, _: &mut Lir, _: &mut Context) -> ExecutionResult {
        todo!()
    }
}
