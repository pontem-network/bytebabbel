use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir2::ir::{Hir2, _Expr};
use crate::bytecode::loc::Loc;

pub struct EventOp(pub usize);

impl InstructionHandler for EventOp {
    fn handle(&self, params: Vec<Loc<_Expr>>, ir: &mut Hir2, ctx: &mut Context) -> ExecutionResult {
        let offset = params[0].clone();
        let len = params[1].clone();
        let topics = params[2..].to_vec();
        ir.log(&ctx.loc, offset, len, topics);
        ExecutionResult::None
    }
}
