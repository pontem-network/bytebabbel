use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir::ir::expression::Expr;
use crate::bytecode::hir::ir::statement::Statement;

#[derive(Debug, Clone)]
pub enum MemoryOp {
    MLoad,
    MStore,
    MStore8,
    MSize,
}

impl InstructionHandler for MemoryOp {
    fn handle(&self, mut params: Vec<Expr>, _: &mut Context) -> ExecutionResult {
        match self {
            MemoryOp::MLoad => {
                let addr = params.remove(0);
                Expr::MLoad {
                    mem_offset: Box::new(addr),
                }
                .into()
            }
            MemoryOp::MStore => {
                let var = params.remove(1);
                let addr = params.remove(0);
                Statement::MemStore { addr, var }.into()
            }
            MemoryOp::MStore8 => {
                let var = params.remove(1);
                let addr = params.remove(0);
                Statement::MemStore8 { addr, var }.into()
            }
            MemoryOp::MSize => Expr::MSize.into(),
        }
    }
}
