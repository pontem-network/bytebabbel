use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir2::ir::expression::Expr;
use primitive_types::U256;
use std::rc::Rc;

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
    fn handle(&self, _: Vec<Rc<Expr>>, ctx: &mut Context) -> ExecutionResult {
        match self {
            CodeOp::CodeSize => {
                ExecutionResult::Expr(vec![Rc::new(Expr::Val(U256::from(ctx.code_size())))])
            }
            CodeOp::CallDataCopy => {
                todo!()
            }
            CodeOp::CodeCopy => {
                panic!("CodeCopy not supported!");
            }
            CodeOp::ExtCodeSize => {
                todo!()
            }
            CodeOp::ExtCodeCopy => {
                todo!()
            }
            CodeOp::ReturnDataSize => {
                todo!()
            }
            CodeOp::ReturnDataCopy => {
                todo!()
            }
            CodeOp::ExtCodeHash => {
                todo!()
            }
            CodeOp::PC => {
                todo!()
            }
            CodeOp::Create => {
                todo!()
            }
            CodeOp::Create2 => {
                todo!()
            }
        }
    }
}
