use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir2::ir::{Expr, Hir2};

pub enum StorageOp {
    SLoad,
    SStore,
}

impl InstructionHandler for StorageOp {
    fn handle(&self, params: Vec<Expr>, ir: &mut Hir2, _: &mut Context) -> ExecutionResult {
        // match self {
        //     StorageOp::SLoad => {
        //         let addr = params[0].clone();
        //         let id = ir.create_var(Expr::SLoad(Box::new(addr)));
        //         ExecutionResult::Output(vec![id])
        //     }
        //     StorageOp::SStore => {
        //         let addr = params[0].clone();
        //         let val = params[1].clone();
        //         ir.sstore(addr, val);
        //         ExecutionResult::None
        //     }
        // }
        todo!()
    }
}
