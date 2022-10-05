use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir2::ir::{Hir2, _Expr};
use crate::bytecode::loc::Loc;

#[derive(Debug, Clone)]
pub enum MemoryOp {
    MLoad,
    MStore,
    MStore8,
    MSize,
}

impl InstructionHandler for MemoryOp {
    fn handle(
        &self,
        mut params: Vec<Loc<_Expr>>,
        ir: &mut Hir2,
        ctx: &mut Context,
    ) -> ExecutionResult {
        match self {
            MemoryOp::MLoad => {
                let addr = Box::new(params.remove(0));
                let id = ir.assign(ctx.loc.wrap(_Expr::MLoad(addr)), &mut ctx.vars);
                ExecutionResult::Output(id.into())
            }
            MemoryOp::MStore => {
                let val = params.remove(1);
                let addr = params.remove(0);
                ir.mstore(&ctx.loc, addr, val);
                ExecutionResult::None
            }
            MemoryOp::MStore8 => {
                let val = params.remove(1);
                let addr = params.remove(0);
                ir.mstore8(&ctx.loc, addr, val);
                ExecutionResult::None
            }
            MemoryOp::MSize => ExecutionResult::Output(_Expr::MSize),
        }
    }
}
