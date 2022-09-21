use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir::ir::var::{Expr, VarId};
use crate::Hir;

#[derive(Debug, Clone)]
pub enum MemoryOp {
    MLoad,
    MStore,
    MStore8,
    MSize,
}

impl InstructionHandler for MemoryOp {
    fn handle(&self, params: Vec<VarId>, ir: &mut Hir, _: &mut Context) -> ExecutionResult {
        match self {
            MemoryOp::MLoad => {
                let addr = params[0];
                let id = ir.create_var(Expr::MLoad(addr));
                ExecutionResult::Output(vec![id])
            }
            MemoryOp::MStore => {
                let addr = params[0];
                let val = params[1];
                ir.mstore(addr, val);
                ExecutionResult::None
            }
            MemoryOp::MStore8 => {
                let addr = params[0];
                let val = params[1];
                ir.mstore8(addr, val);
                ExecutionResult::None
            }
            MemoryOp::MSize => {
                let id = ir.create_var(Expr::MSize);
                ExecutionResult::Output(vec![id])
            }
        }
    }
}
