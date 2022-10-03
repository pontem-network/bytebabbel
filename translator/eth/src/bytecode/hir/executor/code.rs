use primitive_types::U256;

use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir::ir::var::{Expr, VarId};
use crate::{BlockId, Hir};

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
    fn handle(&self, ops: Vec<VarId>, ir: &mut Hir, ctx: &mut Context) -> ExecutionResult {
        match self {
            CodeOp::CodeSize => {
                let id = ir.create_var(Expr::Val(U256::from(ctx.code_size())));
                ExecutionResult::Output(vec![id])
            }
            CodeOp::CallDataCopy => ExecutionResult::Output(vec![]),
            CodeOp::CodeCopy => {
                let offset = ir.resolve_var(ops[1]).unwrap_or_default();
                ir.code_copy(BlockId::from(offset));
                ExecutionResult::Output(vec![])
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
