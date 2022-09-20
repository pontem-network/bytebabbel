mod consts;

use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::ir::Hir2;
use crate::bytecode::hir2::optimizer::consts::constant_fold;

pub fn optimize(hir: Hir2, ctx: &mut Context) -> Hir2 {
    constant_fold(hir, ctx)
}
