use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir2::ir::expression::Expr;
use crate::bytecode::hir2::ir::statement::Statement;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum MemoryOp {
    MLoad,
    MStore,
    MStore8,
    MSize,
}

impl InstructionHandler for MemoryOp {
    fn handle(&self, mut params: Vec<Rc<Expr>>, _: &mut Context) -> ExecutionResult {
        match self {
            MemoryOp::MLoad => {
                let addr = params.remove(0);
                ExecutionResult::Expr(vec![Rc::new(Expr::MLoad { mem_offset: addr })])
            }
            MemoryOp::MStore => {
                let var = params.remove(1);
                let addr = params.remove(0);
                ExecutionResult::Statement(Statement::MemStore { addr, var })
            }
            MemoryOp::MStore8 => {
                let var = params.remove(1);
                let addr = params.remove(0);
                ExecutionResult::Statement(Statement::MemStore8 { addr, var })
            }
            MemoryOp::MSize => ExecutionResult::Expr(vec![Rc::new(Expr::MSize)]),
        }
    }
}
