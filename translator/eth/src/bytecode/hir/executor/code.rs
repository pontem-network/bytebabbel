use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir::ir::{Expr, _Expr};
use crate::Hir;
use primitive_types::U256;

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
    fn handle(&self, _: Vec<Expr>, _: &mut Hir, ctx: &mut Context) -> ExecutionResult {
        match self {
            CodeOp::CodeSize => ExecutionResult::Output(_Expr::Val(U256::from(ctx.code_size()))),
            CodeOp::CallDataCopy => ExecutionResult::None,
            CodeOp::CodeCopy => {
                unimplemented!("CodeCopy")
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
