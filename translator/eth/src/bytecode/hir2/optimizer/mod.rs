mod consts;

use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::ir::Hir2;
use crate::bytecode::hir2::optimizer::consts::const_fold;

pub fn optimize(hir: Hir2, ctx: &mut Context) -> Hir2 {
    const_fold(hir, ctx)
}
