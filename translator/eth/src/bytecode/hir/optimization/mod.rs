use anyhow::Error;

use crate::bytecode::hir::optimization::unused_var::UnusedVarClipper;
use crate::Hir;

mod unused_var;

pub struct IrOptimizer;

impl IrOptimizer {
    pub fn optimize(ir: Hir) -> Result<Hir, Error> {
        let ir = UnusedVarClipper::optimize(ir)?;
        Ok(ir)
    }
}
