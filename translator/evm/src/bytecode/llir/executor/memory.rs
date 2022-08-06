use crate::bytecode::llir::context::Context;
use crate::bytecode::llir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::llir::ir::var::{Var, VarId};
use crate::{Ir, U256};

pub enum MemoryOp {
    MLoad,
    MStore,
    MStore8,
    MSize,
}

impl InstructionHandler for MemoryOp {
    fn handle(&self, params: Vec<VarId>, ir: &mut Ir, context: &mut Context) -> ExecutionResult {
        match self {
            MemoryOp::MLoad => {
                if let Some(addr) = ir.resolve_var(params[0]) {
                    if let Some(val) = context.mem_load(addr) {
                        ExecutionResult::Output(vec![val])
                    } else {
                        let id = ir.create_var(Var::Val(U256::zero()));
                        ExecutionResult::Output(vec![id])
                    }
                } else {
                    todo!("Unaligned memory access");
                }
            }
            MemoryOp::MStore => {
                let val = params[1];
                if let Some(addr) = ir.resolve_var(params[0]) {
                    context.mem_store(addr, val);
                    ir.mem_store(addr, val);
                } else {
                    todo!("Unaligned memory access");
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
