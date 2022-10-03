use crate::bytecode::lir::context::Context;
use crate::bytecode::lir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::lir::ir::{Expr, Lir};

#[derive(Debug, Clone)]
pub enum MemoryOp {
    MLoad,
    MStore,
    MStore8,
    MSize,
}

impl InstructionHandler for MemoryOp {
    fn handle(&self, mut params: Vec<Expr>, ir: &mut Lir, ctx: &mut Context) -> ExecutionResult {
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
