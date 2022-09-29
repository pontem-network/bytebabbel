use crate::bytecode::hir::ir::var::VarId;
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
                let id = ir.assign(ctx.next_var(), Expr::MLoad(addr));
                ExecutionResult::Output(id.into())
            }
            MemoryOp::MStore => {
                let val = params.remove(1);
                let addr = params.remove(0);
                ir.mstore(addr, val);
                ExecutionResult::None
            }
            MemoryOp::MStore8 => {
                // let addr = params[0].clone();
                // let val = params[1].clone();
                // ir.mstore8(addr, val);
                // ExecutionResult::None
                todo!()
            }
            MemoryOp::MSize => {
                // let id = ir.create_var(Expr::MSize);
                // ExecutionResult::Output(vec![id])
                todo!()
            }
        }
    }
}
