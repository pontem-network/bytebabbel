use crate::bytecode::llir::context::Context;
use crate::bytecode::llir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::llir::stack::StackFrame;
use crate::Ir;

pub struct Sha3;

impl InstructionHandler for Sha3 {
    fn handle(&self, _: Vec<StackFrame>, _: &mut Ir, _: &mut Context) -> ExecutionResult {
        todo!()
    }
}

pub struct Address;

impl InstructionHandler for Address {
    fn handle(&self, _: Vec<StackFrame>, _: &mut Ir, _: &mut Context) -> ExecutionResult {
        todo!()
    }
}

pub enum TxMeta {
    Balance,
    Origin,
    Caller,
    CallValue,
    CallDataLoad,
    CallDataSize,
    Blockhash,
    Timestamp,
    GasLimit,
    Difficulty,
    Number,
    GasPrice,
    Coinbase,
    Gas,
}

impl InstructionHandler for TxMeta {
    fn handle(
        &self,
        params: Vec<StackFrame>,
        ir: &mut Ir,
        context: &mut Context,
    ) -> ExecutionResult {
        todo!()
    }
}
