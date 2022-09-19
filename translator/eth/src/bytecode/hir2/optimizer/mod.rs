mod consts;
mod live_time;

use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::ir::Hir2;
use crate::bytecode::hir2::optimizer::consts::constant_fold;
use crate::bytecode::hir2::optimizer::live_time::reduce_live_time;

pub fn optimize(hir: Hir2, ctx: &mut Context) -> Hir2 {
    let ir = constant_fold(hir, ctx);
    reduce_live_time(ir, ctx)
}
