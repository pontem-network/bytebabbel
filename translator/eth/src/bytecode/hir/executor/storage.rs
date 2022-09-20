use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir::ir::expression::Expr;
use crate::bytecode::hir::ir::statement::Statement;

pub enum StorageOp {
    SLoad,
    SStore,
}

impl InstructionHandler for StorageOp {
    fn handle(&self, mut params: Vec<Expr>, _: &mut Context) -> ExecutionResult {
        match self {
            StorageOp::SLoad => {
                let addr = params.remove(0);
                Expr::SLoad {
                    key: Box::new(addr),
                }
                .into()
            }
            StorageOp::SStore => {
                let var = params.remove(1);
                let addr = params.remove(0);
                Statement::SStore { addr, var }.into()
            }
        }
    }
}
