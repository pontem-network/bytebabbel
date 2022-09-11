use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir2::ir::expression::Expr;
use crate::bytecode::hir2::ir::statement::Statement;
use crate::bytecode::hir2::ir::Hir2;

#[derive(Debug, Clone)]
pub enum MemoryOp {
    MLoad,
    MStore,
    MStore8,
    MSize,
}

impl InstructionHandler for MemoryOp {
    fn handle(&self, mut params: Vec<Expr>, ir: &mut Hir2, _: &mut Context) -> ExecutionResult {
        match self {
            MemoryOp::MLoad => {
                let addr = params.remove(0);
                ExecutionResult::Output(vec![Expr::MLoad {
                    mem_offset: Box::new(addr),
                }])
            }
            MemoryOp::MStore => {
                let var = params.remove(1);
                let addr = params.remove(0);
                ir.add_statement(Statement::MemStore { addr, var });
                ExecutionResult::None
            }
            MemoryOp::MStore8 => {
                let var = params.remove(1);
                let addr = params.remove(0);
                ir.add_statement(Statement::MemStore8 { addr, var });
                ExecutionResult::None
            }
            MemoryOp::MSize => ExecutionResult::Output(vec![Expr::MSize]),
        }
    }
}
