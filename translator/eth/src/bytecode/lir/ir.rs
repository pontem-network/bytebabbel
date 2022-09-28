use crate::bytecode::hir::executor::math::{BinaryOp, TernaryOp, UnaryOp};
use crate::bytecode::hir::ir::var::VarId;
use crate::BlockId;
use primitive_types::U256;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Lir {
    labels: HashMap<Label, usize>,
    vars: HashMap<VarId, usize>,
    blocks: Vec<IR>,
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

#[derive(Debug, Clone)]
pub enum Expr {
    Val(U256),
    Var(Box<Expr>),
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
