use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir::ir::var::VarId;
use crate::Hir;

#[derive(Debug, Clone)]
pub enum MemoryOp {
    MLoad,
    MStore,
    MStore8,
    MSize,
}

impl InstructionHandler for MemoryOp {
    fn handle(&self, params: Vec<VarId>, ir: &mut Hir, context: &mut Context) -> ExecutionResult {
        match self {
            MemoryOp::MLoad => {
                if let Some(addr) = ir.resolve_var(params[0]) {
                    if let Some(val) = context.mem_load(addr) {
                        ir.mem_load(addr, val);
                        ExecutionResult::Output(vec![val])
                    } else {
                        todo!("memory load")
                    }
                } else {
                    todo!("Unaligned memory access")
                }
            }
            MemoryOp::MStore => {
                let val = params[1];
                if let Some(addr) = ir.resolve_var(params[0]) {
                    context.mem_store(addr, val);
                    ir.mem_store(addr, val);
                } else {
                    todo!("Unaligned memory access")
                }
                ExecutionResult::None
            }
            MemoryOp::MStore8 => {
                todo!()
            }
            MemoryOp::MSize => {
                todo!()
            }
        }
    }
}
