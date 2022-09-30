use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use anyhow::{anyhow, Error};
use primitive_types::U256;

use crate::bytecode::hir::executor::math::{BinaryOp, TernaryOp, UnaryOp};
use crate::bytecode::mir::ir::types::LocalIndex;

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
        self.resolve_expr(self.inner.get(&id)?)
    }

    pub fn resolve_expr(&self, expr: &Expr) -> Option<U256> {
        match expr {
            Expr::Val(val) => Some(*val),
            Expr::UnaryOp(cmd, arg) => {
                let val = self.resolve_expr(arg)?;
                Some(cmd.calc(val))
            }
            Expr::BinaryOp(cmd, arg1, arg2) => {
                let op1 = self.resolve_expr(arg1)?;
                let op2 = self.resolve_expr(arg2)?;
                Some(cmd.calc(op1, op2))
            }
            Expr::TernaryOp(cmd, arg1, arg2, arg3) => {
                let op1 = self.resolve_expr(arg1)?;
                let op2 = self.resolve_expr(arg2)?;
                let op3 = self.resolve_expr(arg3)?;
                Some(cmd.calc(op1, op2, op3))
            }
            Expr::MLoad(_) => None,
            Expr::SLoad(_) => None,
            Expr::MSize => None,
            Expr::Signer => None,
            Expr::ArgsSize => None,
            Expr::Args(_) => None,
            Expr::Hash(_, _) => None,
            Expr::Var(id) => self.resolve_var(*id),
        }
    }

    pub fn take(&mut self, id: VarId) -> Result<Expr, Error> {
        self.inner
            .remove(&id)
            .ok_or_else(|| anyhow!("VarId not found: {:?}", id))
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Val(U256),
    Var(VarId),
    MLoad(VarId),
    SLoad(VarId),
    Signer,
    MSize,
    ArgsSize,
    Args(VarId),
    UnaryOp(UnaryOp, Box<Expr>),
    BinaryOp(BinaryOp, Box<Expr>, Box<Expr>),
    TernaryOp(TernaryOp, Box<Expr>, Box<Expr>, Box<Expr>),
    Hash(VarId, VarId),
}

#[derive(Hash, Eq, PartialEq, Copy, Clone, Default)]
pub struct VarId(u64);

impl VarId {
    pub fn expr(&self) -> Box<Expr> {
        Box::new(Expr::Var(*self))
    }
}

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
