use crate::bytecode::llir::context::Context;
use crate::bytecode::llir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::llir::ir::var::VarId;
use crate::Ir;

pub enum StorageOp {
    SLoad,
    SStore,
}

impl InstructionHandler for StorageOp {
    fn handle(&self, _: Vec<VarId>, _: &mut Ir, _: &mut Context) -> ExecutionResult {
        todo!()
    }
}
