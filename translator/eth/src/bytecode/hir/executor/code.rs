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
    fn handle(&self, mut params: Vec<Expr>, ir: &mut Hir, ctx: &mut Context) -> ExecutionResult {
        match self {
            CodeOp::CodeSize => ExecutionResult::Output(_Expr::Val(U256::from(ctx.code_size()))),
            CodeOp::CallDataCopy => ExecutionResult::None,
            CodeOp::CodeCopy => {
                let length = params.remove(2);
                let offset = params.remove(1);
                let dest_offset = params.remove(0);

                if let (Some(offset), Some(length)) = (offset.resolve(ctx), length.resolve(ctx)) {
                    let offset = offset.as_u128();
                    let length = length.as_u128();
                    let code = ctx.contract_slice(offset, length).to_vec();
                    ir.code_copy(&ctx.loc, dest_offset, code);
                    ExecutionResult::None
                } else {
                    panic!("unsupported dynamic CodeCopy.")
                }
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
