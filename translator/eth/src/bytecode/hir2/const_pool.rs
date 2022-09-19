use crate::bytecode::hir2::vars::VarId;
use primitive_types::U256;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

#[derive(Default, Clone)]
pub struct ConstPool {
    pool: Rc<RefCell<HashMap<VarId, Const>>>,
}

impl ConstPool {
    pub fn new() -> ConstPool {
        ConstPool {
            pool: Default::default(),
        }
    }

    pub fn assign_val(&mut self, var: VarId, val: U256) {
        let mut pool = self.pool.borrow_mut();
        pool.entry(var).or_insert_with(|| Const::new(val));
    }

    pub fn assign_var(&mut self, var: VarId) {
        let mut pool = self.pool.borrow_mut();
        pool.remove(&var);
    }

    pub fn use_var(&mut self, var: &VarId) {
        let mut pool = self.pool.borrow_mut();
        if let Some(cnt) = pool.get_mut(var) {
            cnt.inc();
        }
    }

    pub fn get_const(&self, var: &VarId) -> Option<Const> {
        let pool = self.pool.borrow();
        pool.get(var).cloned()
    }
}

impl Debug for ConstPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pool = self.pool.borrow();
        write!(f, "{:?}", pool)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Const {
    pub val: U256,
    pub users: usize,
}

impl Const {
    pub fn new(val: U256) -> Const {
        Const {
            val,
            users: Default::default(),
        }
    }

    pub fn inc(&mut self) {
        self.users += 1;
    }
}
