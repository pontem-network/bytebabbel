use crate::bytecode::hir2::ir::expression::Expr;
use crate::bytecode::hir2::ir::VarId;
use std::collections::HashMap;

#[derive(Default, Debug, Clone)]
pub struct Vars {
    inner: HashMap<VarId, Expr>,
}

impl Vars {
    pub fn insert(&mut self, var: VarId, expr: Expr) {
        self.inner.insert(var, expr);
    }

    pub fn get(&self, var: &VarId) -> Option<&Expr> {
        self.inner.get(var)
    }
}
