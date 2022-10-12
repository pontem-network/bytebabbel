use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir::ir::{Expr, _Expr};

use crate::Hir;

pub enum StorageOp {
    SLoad,
    SStore,
}

impl InstructionHandler for StorageOp {
    fn handle(&self, mut params: Vec<Expr>, ir: &mut Hir, ctx: &mut Context) -> ExecutionResult {
        match self {
            StorageOp::SLoad => {
                let addr = Box::new(params.remove(0));
                let id = ir.assign(ctx.loc.wrap(_Expr::SLoad(addr)), &mut ctx.vars);
                ExecutionResult::Output(id.into())
            }
            StorageOp::SStore => {
                let val = params.remove(1);
                let addr = params.remove(0);
                ir.sstore(&ctx.loc, addr, val);
                ExecutionResult::None
            }
        }
    }
}
