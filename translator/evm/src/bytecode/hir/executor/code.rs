use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir::ir::var::{Var, VarId};
use crate::{BlockId, Hir, U256};

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
    fn handle(&self, ops: Vec<VarId>, ir: &mut Hir, _: &mut Context) -> ExecutionResult {
        match self {
            CodeOp::CodeSize => {
                todo!()
            }
            CodeOp::CallDataCopy => {
                let id = ir.create_var(Var::Val(U256::from(42)));
                ExecutionResult::Output(vec![id])
            }
            CodeOp::CodeCopy => {
                let offset = ir.resolve_var(ops[1]).unwrap_or_default();
                ExecutionResult::CodeCopy(BlockId::from(offset.as_usize()))
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
