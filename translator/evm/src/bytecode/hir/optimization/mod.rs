mod unused_var;

use crate::bytecode::hir::optimization::unused_var::UnusedVarClipper;
use crate::Hir;
use anyhow::Error;

pub struct IrOptimizer;

impl IrOptimizer {
    pub fn optimize(ir: Hir) -> Result<Hir, Error> {
        let ir = UnusedVarClipper::optimize(ir)?;
        Ok(ir)
    }
}
