use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir2::ir::expression::Expr;
use crate::bytecode::hir2::ir::statement::Statement;
use std::rc::Rc;

pub enum StorageOp {
    SLoad,
    SStore,
}

impl InstructionHandler for StorageOp {
    fn handle(&self, mut params: Vec<Rc<Expr>>, _: &mut Context) -> ExecutionResult {
        match self {
            StorageOp::SLoad => {
                let addr = params.remove(0);
                ExecutionResult::Expr(vec![Rc::new(Expr::SLoad { key: addr })])
            }
            StorageOp::SStore => {
                let var = params.remove(1);
                let addr = params.remove(0);
                ExecutionResult::Statement(Statement::SStore { addr, var })
            }
        }
    }
}
