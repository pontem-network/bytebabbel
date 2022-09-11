use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir2::ir::expression::Expr;
use crate::bytecode::hir2::ir::statement::Statement;
use crate::bytecode::hir2::ir::Hir2;

pub struct EventOp(pub usize);

impl InstructionHandler for EventOp {
    fn handle(&self, mut params: Vec<Expr>, ir: &mut Hir2, _: &mut Context) -> ExecutionResult {
        let len = params.remove(1);
        let offset = params.remove(0);
        let topics = params;
        ir.add_statement(Statement::Log {
            offset,
            len,
            topics,
        });
        ExecutionResult::None
    }
}
