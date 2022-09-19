mod consts;

use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::ir::Hir2;
use anyhow::Error;

pub fn optimize(hir: Hir2, ctx: &mut Context) -> Result<Hir2, Error> {
    let hir = const_fold(hir, ctx)?;
    Ok(hir)
}
