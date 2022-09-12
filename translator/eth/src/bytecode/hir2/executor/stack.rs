use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir2::ir::expression::Expr;
use primitive_types::U256;
use std::rc::Rc;

pub enum StackOp {
    Push(Vec<u8>),
    Dup(usize),
    Swap(usize),
    Pop,
}

impl InstructionHandler for StackOp {
    fn handle(&self, mut params: Vec<Rc<Expr>>, _: &mut Context) -> ExecutionResult {
        match self {
            StackOp::Push(val) => {
                ExecutionResult::Expr(vec![Rc::new(Expr::Val(U256::from(val.as_slice())))])
            }
            StackOp::Dup(_) => {
                let new_item = params[params.len() - 1].clone();
                params.insert(0, new_item);
                ExecutionResult::Expr(params)
            }
            StackOp::Swap(_) => {
                let last_index = params.len() - 1;
                params.swap(0, last_index);
                ExecutionResult::Expr(params)
            }
            StackOp::Pop => ExecutionResult::None,
        }
    }
}
