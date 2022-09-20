mod consts;

use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::ir::Hir;
use crate::bytecode::hir::optimizer::consts::constant_fold;

pub fn optimize(hir: Hir, ctx: &mut Context) -> Hir {
    constant_fold(hir, ctx)
}
