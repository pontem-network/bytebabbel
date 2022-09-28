use crate::bytecode::hir::context::Context;
use crate::bytecode::lir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::lir::ir::{Expr, Lir};

pub struct EventOp(pub usize);

impl InstructionHandler for EventOp {
    fn handle(&self, params: Vec<Expr>, ir: &mut Lir, _: &mut Context) -> ExecutionResult {
        let offset = params[0].clone();
        let len = params[1].clone();
        let topics = params[2..].to_vec();
        // ir.log(offset, len, topics);
        ExecutionResult::None
    }
}
