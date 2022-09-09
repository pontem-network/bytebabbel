use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir::ir::var::VarId;
use crate::Hir;

pub struct EventOp(pub usize);

impl InstructionHandler for EventOp {
    fn handle(&self, params: Vec<VarId>, ir: &mut Hir, _: &mut Context) -> ExecutionResult {
        let offset = params[0];
        let len = params[1];
        let topics = params[2..].to_vec();
        ir.log(offset, len, topics);
        ExecutionResult::None
    }
}
