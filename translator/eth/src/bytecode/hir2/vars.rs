use crate::bytecode::hir2::ir::{Expr, VarId};
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Default, Clone)]
pub struct Vars {
    vars: HashMap<VarId, Expr>,
    var_seq: Rc<Cell<u32>>,
}

impl Vars {
    pub fn gen_tmp(&self) -> VarId {
        let idx = self.var_seq.get();
        let var = VarId::new_tmp(idx);
        self.var_seq.set(idx + 1);
        var
    }

    pub fn get(&self, var: &VarId) -> Option<&Expr> {
        self.vars.get(var)
    }

    pub fn set(&mut self, var: VarId, expr: Expr) {
        self.vars.insert(var, expr);
    }
}
