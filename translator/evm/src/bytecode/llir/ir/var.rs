use crate::bytecode::llir::executor::math::{BinaryOp, TernaryOp, UnaryOp};
use crate::bytecode::types::U256;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

#[derive(Default, Debug)]
pub struct Vars {
    inner: HashMap<VarId, Var>,
}

impl Vars {
    pub fn create(&mut self, var: Var) -> VarId {
        let id = VarId(self.inner.len() as u64);
        self.inner.insert(id, var);
        id
    }

    pub fn set_val(&mut self, id: VarId, var: Var) {
        self.inner.insert(id, var);
    }

    pub fn get(&self, id: &VarId) -> &Var {
        self.inner.get(id).unwrap()
    }

    pub fn resolve_var(&self, id: VarId) -> Option<U256> {
        match self.inner.get(&id) {
            Some(Var::Val(val)) => Some(val.clone()),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum Var {
    Val(U256),
    Param(u16),
    UnaryOp(UnaryOp, VarId),
    BinaryOp(BinaryOp, VarId, VarId),
    TernaryOp(TernaryOp, VarId, VarId, VarId),
}

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct VarId(u64);

impl Into<u64> for VarId {
    fn into(self) -> u64 {
        self.0
    }
}

impl AsRef<u64> for VarId {
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

impl Debug for VarId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "var_{}", self.0)
    }
}
