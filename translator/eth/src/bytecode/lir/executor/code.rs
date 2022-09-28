use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::ir::var::VarId;
use crate::bytecode::lir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::lir::ir::{Expr, Lir};
use crate::BlockId;
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
    fn handle(&self, ops: Vec<Expr>, ir: &mut Lir, ctx: &mut Context) -> ExecutionResult {
        match self {
            CodeOp::CodeSize => {
                // let id = ir.create_var(Expr::Val(U256::from(ctx.code_size())));
                ExecutionResult::Output(vec![Expr::Val(U256::from(ctx.code_size()))])
            }
            CodeOp::CallDataCopy => ExecutionResult::Output(vec![]),
            CodeOp::CodeCopy => {
                // let offset = ir.resolve_var(ops[1].clone()).unwrap_or_default();
                // ir.code_copy(BlockId::from(offset));
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
