use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir2::ir::expression::Expr;
use crate::bytecode::hir2::ir::statement::Statement;
use crate::bytecode::hir2::ir::Hir2;

pub enum StorageOp {
    SLoad,
    SStore,
}

impl InstructionHandler for StorageOp {
    fn handle(&self, mut params: Vec<Expr>, ir: &mut Hir2, _: &mut Context) -> ExecutionResult {
        match self {
            StorageOp::SLoad => {
                let addr = params.remove(0);
                ExecutionResult::Output(vec![Expr::SLoad {
                    key: Box::new(addr),
                }])
            }
            StorageOp::SStore => {
                let var = params.remove(1);
                let addr = params.remove(0);
                ir.add_statement(Statement::SStore { addr, var });
                ExecutionResult::None
            }
        }
    }
}
