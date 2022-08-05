use crate::bytecode::llir::context::Context;
use crate::bytecode::llir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::llir::ir::var::VarId;
use crate::Ir;

pub struct EventOp(pub usize);

impl InstructionHandler for EventOp {
    fn handle(&self, _: Vec<VarId>, _: &mut Ir, _: &mut Context) -> ExecutionResult {
        todo!()
    }
}
