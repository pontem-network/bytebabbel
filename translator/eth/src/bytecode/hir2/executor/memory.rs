use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir2::ir::{Expr, Hir2};

#[derive(Debug, Clone)]
pub enum MemoryOp {
    MLoad,
    MStore,
    MStore8,
    MSize,
}

impl InstructionHandler for MemoryOp {
    fn handle(&self, mut params: Vec<Expr>, ir: &mut Hir2, ctx: &mut Context) -> ExecutionResult {
        match self {
            MemoryOp::MLoad => {
                let addr = Box::new(params.remove(0));
                let id = ir.assign(Expr::MLoad(addr), ctx);
                ExecutionResult::Output(id.into())
            }
            MemoryOp::MStore => {
                let val = params.remove(1);
                let addr = params.remove(0);
                ir.mstore(addr, val);
                ExecutionResult::None
            }
            MemoryOp::MStore8 => {
                let val = params.remove(1);
                let addr = params.remove(0);
                ir.mstore8(addr, val);
                ExecutionResult::None
            }
            MemoryOp::MSize => ExecutionResult::Output(Expr::MSize),
        }
    }
}
