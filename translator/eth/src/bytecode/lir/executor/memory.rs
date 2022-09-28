use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::ir::var::VarId;
use crate::bytecode::lir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::lir::ir::{Expr, Lir};

#[derive(Debug, Clone)]
pub enum MemoryOp {
    MLoad,
    MStore,
    MStore8,
    MSize,
}

impl InstructionHandler for MemoryOp {
    fn handle(&self, params: Vec<Expr>, ir: &mut Lir, _: &mut Context) -> ExecutionResult {
        // match self {
        //     MemoryOp::MLoad => {
        //         let addr = params[0].clone();
        //         let id = ir.create_var(Expr::MLoad(Box::new(addr)));
        //         ExecutionResult::Output(vec![id])
        //     }
        //     MemoryOp::MStore => {
        //         let addr = params[0].clone();
        //         let val = params[1].clone();
        //         ir.mstore(addr, val);
        //         ExecutionResult::None
        //     }
        //     MemoryOp::MStore8 => {
        //         let addr = params[0].clone();
        //         let val = params[1].clone();
        //         ir.mstore8(addr, val);
        //         ExecutionResult::None
        //     }
        //     MemoryOp::MSize => {
        //         let id = ir.create_var(Expr::MSize);
        //         ExecutionResult::Output(vec![id])
        //     }
        // }
        todo!()
    }
}
