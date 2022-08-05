use crate::bytecode::llir::context::Context;
use crate::bytecode::llir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::llir::ir::var::VarId;
use crate::Ir;

pub enum CodeOp {
    CodeSize,
    CallDataCopy,
    CodeCopy,
    ExtCodeSize,
    ExtCodeCopy,
    ReturnDataSize,
    ReturnDataCopy,
    ExtCodeHash,
    PC,
    Create,
    Create2,
}

impl InstructionHandler for CodeOp {
    fn handle(&self, _: Vec<VarId>, _: &mut Ir, _: &mut Context) -> ExecutionResult {
        todo!()
    }
}
