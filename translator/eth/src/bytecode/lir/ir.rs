use crate::bytecode::hir::ir::var::VarId;
use crate::bytecode::lir::executor::math::{BinaryOp, TernaryOp, UnaryOp};
use crate::BlockId;
use primitive_types::U256;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Lir {
    labels: HashMap<Label, usize>,
    statement: Vec<IR>,
}

#[derive(Debug, Clone)]
pub enum IR {
    Label(Label),
    Assign(VarId, Expr),
    MemStore8 {
        addr: Expr,
        var: Expr,
    },
    MemStore {
        addr: Expr,
        var: Expr,
    },
    SStore {
        addr: Expr,
        var: Expr,
    },
    Log {
        offset: Expr,
        len: Expr,
        topics: Vec<Expr>,
    },
    Stop,
    Abort(u8),
    Result {
        offset: Expr,
        len: Expr,
    },
    GoTo(Label),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Expr {
    Val(U256),
    Var(VarId),
    MLoad(Box<Expr>),
    SLoad(Box<Expr>),
    Signer,
    MSize,
    ArgsSize,
    Args(Box<Expr>),
    UnaryOp(UnaryOp, Box<Expr>),
    BinaryOp(BinaryOp, Box<Expr>, Box<Expr>),
    TernaryOp(TernaryOp, Box<Expr>, Box<Expr>, Box<Expr>),
    Hash(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Label {
    from: BlockId,
    to: BlockId,
}

impl Lir {
    pub fn assign(&mut self, var: VarId, expr: Expr) -> VarId {
        self.statement.push(IR::Assign(var, expr));
        var
    }

    pub fn abort(&mut self, code: u8) {
        self.statement.push(IR::Abort(code));
    }

    pub fn result(&mut self, offset: Expr, len: Expr) {
        self.statement.push(IR::Result { offset, len });
    }

    pub fn stop(&mut self) {
        self.statement.push(IR::Stop);
    }

    pub fn mstore(&mut self, addr: Expr, var: Expr) {
        self.statement.push(IR::MemStore { addr, var });
    }
}

impl From<U256> for Expr {
    fn from(val: U256) -> Self {
        Expr::Val(val)
    }
}

impl From<u128> for Expr {
    fn from(val: u128) -> Self {
        Expr::Val(U256::from(val))
    }
}

impl From<VarId> for Expr {
    fn from(id: VarId) -> Self {
        Expr::Var(id)
    }
}
