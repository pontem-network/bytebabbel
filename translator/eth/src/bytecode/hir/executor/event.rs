use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir::ir::var::VarId;
use crate::Hir;

pub struct EventOp(pub usize);

impl InstructionHandler for EventOp {
    fn handle(&self, _: Vec<VarId>, _: &mut Hir, _: &mut Context) -> ExecutionResult {
        todo!()
    }
}
