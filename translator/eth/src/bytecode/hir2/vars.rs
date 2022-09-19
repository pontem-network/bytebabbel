use crate::bytecode::hir2::ir::expression::Expr;
use crate::bytecode::tracing::exec::StackItem;
use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq, Copy, Clone, Default, Debug, Ord, PartialOrd)]
pub struct VarId(u64);

impl VarId {
    pub fn index(&self) -> u64 {
        self.0
    }
}

impl From<u64> for VarId {
    fn from(index: u64) -> Self {
        VarId(index)
    }
}

#[derive(Default, Debug, Clone)]
pub struct Vars {
    inner: HashMap<VarId, (Expr, StackItem)>,
    seq: u64,
}

impl Vars {
    pub fn insert(&mut self, expr: Expr, stack_item: StackItem) -> VarId {
        self.seq += 1;
        let id = VarId(self.seq);
        self.inner.insert(id, (expr, stack_item));
        id
    }

    pub fn get(&self, var: &VarId) -> Option<(Expr, StackItem)> {
        self.inner.get(var).cloned()
    }
}
