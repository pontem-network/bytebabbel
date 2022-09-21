use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir::ir::var::{Expr, VarId};
use crate::Hir;

pub enum StorageOp {
    SLoad,
    SStore,
}

impl InstructionHandler for StorageOp {
    fn handle(&self, params: Vec<VarId>, ir: &mut Hir, _: &mut Context) -> ExecutionResult {
        match self {
            StorageOp::SLoad => {
                let addr = params[0];
                let id = ir.create_var(Expr::SLoad(addr));
                ExecutionResult::Output(vec![id])
            }
            StorageOp::SStore => {
                let addr = params[0];
                let val = params[1];
                ir.sstore(addr, val);
                ExecutionResult::None
            }
        }
    }
}
