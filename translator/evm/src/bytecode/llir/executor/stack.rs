use crate::bytecode::llir::context::Context;
use crate::bytecode::llir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::llir::ir::var::{Var, VarId};
use crate::{Ir, U256};

pub enum StackOp {
    Push(Vec<u8>),
    Dup(usize),
    Swap(usize),
    Pop,
}

impl InstructionHandler for StackOp {
    fn handle(&self, mut params: Vec<VarId>, ir: &mut Ir, _: &mut Context) -> ExecutionResult {
        match self {
            StackOp::Push(val) => {
                let id = ir.create_var(Var::Val(U256::from(val.as_slice())));
                ExecutionResult::Output(vec![id])
            }
            StackOp::Dup(_) => {
                let new_item = params[params.len() - 1];
                params.insert(0, new_item);
                ExecutionResult::Output(params)
            }
            StackOp::Swap(_) => {
                let last_index = params.len() - 1;
                params.swap(0, last_index);
                ExecutionResult::Output(params)
            }
            StackOp::Pop => ExecutionResult::None,
        }
    }
}
