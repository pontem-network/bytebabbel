use crate::bytecode::hir::executor::math::{BinaryOp, TernaryOp, UnaryOp};
use anyhow::{anyhow, Error};
use primitive_types::U256;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

#[derive(Default, Debug)]
pub struct Vars {
    inner: HashMap<VarId, Eval>,
}

impl Vars {
    pub fn create(&mut self, var: Eval) -> VarId {
        let id = VarId(self.inner.len() as u64);
        self.inner.insert(id, var);
        id
    }

    pub fn set_val(&mut self, id: VarId, var: Eval) {
        self.inner.insert(id, var);
    }

    pub fn get(&self, id: &VarId) -> &Eval {
        self.inner.get(id).unwrap()
    }

    pub fn resolve_var(&self, id: VarId) -> Option<U256> {
        match self.inner.get(&id) {
            Some(Eval::Val(val)) => Some(*val),
            None => None,
            Some(Eval::UnaryOp(cmd, op)) => {
                let val = self.resolve_var(*op)?;
                Some(cmd.calc(val))
            }
            Some(Eval::BinaryOp(cmd, op1, op2)) => {
                let op1 = self.resolve_var(*op1)?;
                let op2 = self.resolve_var(*op2)?;
                Some(cmd.calc(op1, op2))
            }
            Some(Eval::TernaryOp(cmd, op1, op2, op3)) => {
                let op1 = self.resolve_var(*op1)?;
                let op2 = self.resolve_var(*op2)?;
                let op3 = self.resolve_var(*op3)?;
                Some(cmd.calc(op1, op2, op3))
            }
            Some(Eval::MLoad(_)) => None,
            Some(Eval::SLoad(_)) => None,
            Some(Eval::MSize) => None,
            Some(Eval::Signer) => None,
            Some(Eval::ArgsSize) => None,
            Some(Eval::Args(_)) => None,
            Some(Eval::Hash(_, _)) => None,
        }
    }

    pub fn take(&mut self, id: VarId) -> Result<Eval, Error> {
        self.inner
            .remove(&id)
            .ok_or_else(|| anyhow!("VarId not found: {:?}", id))
    }
}

#[derive(Debug)]
pub enum Eval {
    Val(U256),
    MLoad(VarId),
    SLoad(VarId),
    Signer,
    MSize,
    ArgsSize,
    Args(VarId),
    UnaryOp(UnaryOp, VarId),
    BinaryOp(BinaryOp, VarId, VarId),
    TernaryOp(TernaryOp, VarId, VarId, VarId),
    Hash(VarId, VarId),
}

#[derive(Hash, Eq, PartialEq, Copy, Clone, Default)]
pub struct VarId(u64);

impl From<VarId> for u64 {
    fn from(id: VarId) -> u64 {
        id.0
    }
}

impl From<u64> for VarId {
    fn from(id: u64) -> VarId {
        VarId(id)
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
