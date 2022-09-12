use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir2::ir::expression::Expr;
use crate::bytecode::hir2::ir::statement::Statement;
use std::rc::Rc;

pub struct EventOp(pub usize);

impl InstructionHandler for EventOp {
    fn handle(&self, mut params: Vec<Rc<Expr>>, _: &mut Context) -> ExecutionResult {
        let len = params.remove(1);
        let offset = params.remove(0);
        let topics = params;
        ExecutionResult::Statement(Statement::Log {
            offset,
            len,
            topics,
        })
    }
}