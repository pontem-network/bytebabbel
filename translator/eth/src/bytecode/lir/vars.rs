use crate::bytecode::hir::ir::var::VarId;
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Default, Clone)]
pub struct Vars {
    vars: HashMap<VarId, usize>,
    var_seq: Rc<Cell<u64>>,
}

impl Vars {
    pub fn next_var(&self) -> VarId {
        let var = VarId::from(self.var_seq.get());
        self.var_seq.set(self.var_seq.get() + 1);
        var
    }

    pub fn get(&self, var: &VarId) -> Option<usize> {
        self.vars.get(var).cloned()
    }

    pub fn set(&mut self, var: VarId, expr: usize) {
        self.vars.insert(var, expr);
    }
}
