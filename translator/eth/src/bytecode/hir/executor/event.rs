use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir::ir::expression::Expr;
use crate::bytecode::hir::ir::statement::Statement;

pub struct EventOp(pub usize);

impl InstructionHandler for EventOp {
    fn handle(&self, mut params: Vec<Expr>, _: &mut Context) -> ExecutionResult {
        let len = params.remove(1);
        let offset = params.remove(0);
        let topics = params;
        Statement::Log {
            offset,
            len,
            topics,
        }
        .into()
    }
}
