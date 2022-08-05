use crate::bytecode::llir::context::Context;
use crate::bytecode::llir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::llir::stack::StackFrame;
use crate::Ir;

pub enum StorageOp {
    SLoad,
    SStore,
}

impl InstructionHandler for StorageOp {
    fn handle(
        &self,
        params: Vec<StackFrame>,
        ir: &mut Ir,
        context: &mut Context,
    ) -> ExecutionResult {
        todo!()
    }
}
