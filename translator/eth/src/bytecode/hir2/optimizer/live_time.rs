use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::ir::Hir2;

pub fn reduce_live_time(hir: Hir2, ctx: &mut Context) -> Hir2 {
    let statements = hir.inner();
    for st in &statements {}

    statements.into()
}
