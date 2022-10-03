use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir2::ir::{Expr, Hir2};

pub struct EventOp(pub usize);

impl InstructionHandler for EventOp {
    fn handle(&self, params: Vec<Expr>, ir: &mut Hir2, _: &mut Context) -> ExecutionResult {
        let offset = params[0].clone();
        let len = params[1].clone();
        let topics = params[2..].to_vec();
        // ir.log(offset, len, topics);
        ExecutionResult::None
    }
}
