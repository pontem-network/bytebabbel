use crate::bytecode::hir2::ir::expression::Expr;
use crate::bytecode::tracing::exec::StackItem;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Hash, Eq, PartialEq, Copy, Clone, Default, Debug, Ord, PartialOrd)]
pub struct VarId(u64);

impl VarId {
    pub fn index(&self) -> u64 {
        self.0
    }
}

#[derive(Default, Debug, Clone)]
pub struct Vars {
    inner: HashMap<VarId, (Rc<Expr>, StackItem)>,
    seq: u64,
}

impl Vars {
    pub fn insert(&mut self, expr: Rc<Expr>, stack_item: StackItem) -> VarId {
        self.seq += 1;
        let id = VarId(self.seq);
        self.inner.insert(id, (expr, stack_item));
        id
    }

    pub fn get(&self, var: &VarId) -> Option<(Rc<Expr>, StackItem)> {
        self.inner.get(var).cloned()
    }
}
