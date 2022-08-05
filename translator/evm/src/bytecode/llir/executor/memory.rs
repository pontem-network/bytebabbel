use crate::bytecode::llir::context::Context;
use crate::bytecode::llir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::llir::stack::StackFrame;
use crate::Ir;

pub enum MemoryOp {
    MLoad,
    MStore,
    MStore8,
    MSize,
}

impl InstructionHandler for MemoryOp {
    fn handle(&self, params: Vec<StackFrame>, ir: &mut Ir, context: &mut Context) -> ExecutionResult {
        todo!()
    }
}