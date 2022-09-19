use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir2::ir::expression::Expr;

pub enum CallOp {
    Call,
    CallCode,
    DelegateCall,
    StaticCall,
}

impl InstructionHandler for CallOp {
    fn handle(&self, _: Vec<Expr>, _: &mut Context) -> ExecutionResult {
        todo!()
    }
}
