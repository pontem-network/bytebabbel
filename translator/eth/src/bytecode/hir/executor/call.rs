use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir::ir::_Expr;
use crate::bytecode::loc::Loc;
use crate::Hir;

pub enum CallOp {
    Call,
    CallCode,
    DelegateCall,
    StaticCall,
}

impl InstructionHandler for CallOp {
    fn handle(&self, _: Vec<Loc<_Expr>>, _: &mut Hir, _: &mut Context) -> ExecutionResult {
        todo!()
    }
}
