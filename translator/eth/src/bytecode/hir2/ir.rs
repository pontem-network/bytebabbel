use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::debug::print_stmt;
use crate::bytecode::hir2::executor::math::{BinaryOp, TernaryOp, UnaryOp};
use crate::bytecode::hir2::vars::Vars;
use crate::BlockId;
use anyhow::Error;
use primitive_types::U256;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{Debug, Display, Write};

#[derive(Debug, Clone, Default)]
pub struct Hir2 {
    labels: HashMap<Label, usize>,
    statement: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Label(Label),
    StoreContext(BTreeMap<VarId, Expr>),
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
    BrunchTrue(Expr, Label),
    Brunch(Label),
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
    Copy(Box<Expr>),
}

impl Expr {
    pub fn resolve(&self, ir: &Hir2, ctx: &Context) -> Option<U256> {
        match self {
            Expr::Val(val) => Some(*val),
            Expr::Var(var) => {
                let expr = ctx.vars.get(var)?;
                expr.resolve(ir, ctx)
            }
            Expr::MLoad(_) => None,
            Expr::SLoad(_) => None,
            Expr::Signer => None,
            Expr::MSize => None,
            Expr::ArgsSize => None,
            Expr::Args(_) => None,
            Expr::UnaryOp(cnd, arg) => {
                let arg = arg.resolve(ir, ctx)?;
                Some(cnd.calc(arg))
            }
            Expr::BinaryOp(cnd, arg1, arg2) => {
                let arg1 = arg1.resolve(ir, ctx)?;
                let arg2 = arg2.resolve(ir, ctx)?;
                Some(cnd.calc(arg1, arg2))
            }
            Expr::TernaryOp(cnd, arg1, arg2, arg3) => {
                let arg1 = arg1.resolve(ir, ctx)?;
                let arg2 = arg2.resolve(ir, ctx)?;
                let arg3 = arg3.resolve(ir, ctx)?;
                Some(cnd.calc(arg1, arg2, arg3))
            }
            Expr::Hash(_, _) => None,
            Expr::Copy(expr) => expr.resolve(ir, ctx),
        }
    }

    pub fn is_var(&self) -> bool {
        matches!(self, Expr::Var(_))
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Label {
    pub to: BlockId,
}

impl From<BlockId> for Label {
    fn from(to: BlockId) -> Self {
        Self { to }
    }
}

impl Hir2 {
    pub fn assign(&mut self, expr: Expr, vars: &mut Vars) -> VarId {
        let var = vars.gen_tmp();
        self.statement.push(Stmt::Assign(var, expr.clone()));
        vars.set(var, expr.clone());
        var
    }

    pub fn abort(&mut self, code: u8) {
        self.statement.push(Stmt::Abort(code));
    }

    pub fn result(&mut self, offset: Expr, len: Expr) {
        self.statement.push(Stmt::Result { offset, len });
    }

    pub fn stop(&mut self) {
        self.statement.push(Stmt::Stop);
    }

    pub fn return_(&mut self, offset: Expr, len: Expr) {
        self.statement.push(Stmt::Result { offset, len });
    }

    pub fn mstore(&mut self, addr: Expr, var: Expr) {
        self.statement.push(Stmt::MemStore { addr, var });
    }

    pub fn mstore8(&mut self, addr: Expr, var: Expr) {
        self.statement.push(Stmt::MemStore8 { addr, var });
    }

    pub fn save_context(&mut self, context: BTreeMap<VarId, Expr>) {
        self.statement.push(Stmt::StoreContext(context));
    }

    pub fn true_brunch(&mut self, cnd: Expr, label: Label) {
        self.statement.push(Stmt::BrunchTrue(cnd, label));
    }

    pub fn label(&mut self, label: Label) {
        self.statement.push(Stmt::Label(label));
    }

    pub fn print<B: Write>(&self, buf: &mut B) -> Result<(), Error> {
        for stmt in &self.statement {
            print_stmt(buf, &stmt)?;
        }
        Ok(())
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

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Ord, PartialOrd)]
pub struct VarId(u32, bool);

impl VarId {
    pub fn new_var(idx: u32) -> Self {
        VarId(idx, false)
    }

    pub fn new_tmp(idx: u32) -> Self {
        VarId(idx, true)
    }
}

impl Display for VarId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.1 {
            write!(f, "tmp{}", self.0)
        } else {
            write!(f, "var{}", self.0)
        }
    }
}
