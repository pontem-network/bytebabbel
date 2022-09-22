use crate::bytecode::hir::executor::math::{BinaryOp, TernaryOp, UnaryOp};
use crate::bytecode::mir::ir::types::LocalIndex;
use anyhow::{anyhow, Error};
use primitive_types::U256;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

#[derive(Default, Debug)]
pub struct Vars {
    inner: HashMap<VarId, Expr>,
}

impl Vars {
    pub fn create(&mut self, var: Expr) -> VarId {
        let id = VarId(self.inner.len() as u64);
        self.inner.insert(id, var);
        id
    }

    pub fn set_val(&mut self, id: VarId, var: Expr) {
        self.inner.insert(id, var);
    }

    pub fn get(&self, id: &VarId) -> &Expr {
        self.inner.get(id).unwrap()
    }

    pub fn resolve_var(&self, id: VarId) -> Option<U256> {
        match self.inner.get(&id) {
            Some(Expr::Val(val)) => Some(*val),
            None => None,
            Some(Expr::UnaryOp(cmd, op)) => {
                let val = self.resolve_var(*op)?;
                Some(cmd.calc(val))
            }
            Some(Expr::BinaryOp(cmd, op1, op2)) => {
                let op1 = self.resolve_var(*op1)?;
                let op2 = self.resolve_var(*op2)?;
                Some(cmd.calc(op1, op2))
            }
            Some(Expr::TernaryOp(cmd, op1, op2, op3)) => {
                let op1 = self.resolve_var(*op1)?;
                let op2 = self.resolve_var(*op2)?;
                let op3 = self.resolve_var(*op3)?;
                Some(cmd.calc(op1, op2, op3))
            }
            Some(Expr::MLoad(_)) => None,
            Some(Expr::SLoad(_)) => None,
            Some(Expr::MSize) => None,
            Some(Expr::Signer) => None,
            Some(Expr::ArgsSize) => None,
            Some(Expr::Args(_)) => None,
            Some(Expr::Hash(_, _)) => None,
        }
    }

    pub fn take(&mut self, id: VarId) -> Result<Expr, Error> {
        self.inner
            .remove(&id)
            .ok_or_else(|| anyhow!("VarId not found: {:?}", id))
    }
}

#[derive(Debug)]
pub enum Expr {
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

impl VarId {
    pub fn local_index(&self) -> LocalIndex {
        self.0 as u8
    }
}

impl Debug for VarId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "var_{}", self.0)
    }
}
