use crate::bytecode::llir::context::Context;
use crate::bytecode::llir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::llir::stack::StackFrame;
use crate::Ir;

pub enum ControlFlow {
    Stop,
    Return,
    Revert,
    Abort(u8),
    Jump,
    JumpIf,
}

impl InstructionHandler for ControlFlow {
    fn handle(
        &self,
        params: Vec<StackFrame>,
        ir: &mut Ir,
        context: &mut Context,
    ) -> ExecutionResult {
        todo!()
    }
}

pub fn abort(code: u8) -> ExecutionResult {
    ExecutionResult::Abort(code)
}

pub fn stop() -> ExecutionResult {
    ExecutionResult::Stop
}

pub fn revert(mut params: Vec<StackFrame>) -> ExecutionResult {
    let len = params.remove(1);
    let offset = params.remove(0);
    ExecutionResult::Result {
        offset,
        len,
        revert: true,
    }
}

pub fn res(mut params: Vec<StackFrame>) -> ExecutionResult {
    let len = params.remove(1);
    let offset = params.remove(0);
    ExecutionResult::Result {
        offset,
        len,
        revert: false,
    }
}
