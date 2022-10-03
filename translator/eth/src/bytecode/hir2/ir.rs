use crate::bytecode::hir::ir::var::VarId;
use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::executor::math::{BinaryOp, TernaryOp, UnaryOp};
use crate::BlockId;
use primitive_types::U256;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Hir2 {
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

impl Expr {
    pub fn resolve(&self, ir: &Hir2, ctx: &Context) -> Option<U256> {
        match self {
            Expr::Val(val) => Some(*val),
            Expr::Var(var) => {
                let expr = ir.get_var(*var, ctx);
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
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Label {
    from: BlockId,
    to: BlockId,
}

impl Hir2 {
    pub fn assign(&mut self, expr: Expr, ctx: &mut Context) -> VarId {
        let var = ctx.vars.next_var();
        let ixd = self.statement.len();
        self.statement.push(IR::Assign(var, expr));
        ctx.vars.set(var, ixd);
        var
    }

    pub fn get_var(&self, var: VarId, ctx: &Context) -> &Expr {
        let assign_idx = ctx.vars.get(&var).expect("var not found");
        if let IR::Assign(var_id, stmt) = &self.statement[assign_idx] {
            assert_eq!(*var_id, var);
            stmt
        } else {
            panic!("invalid var assignment");
        }
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

    pub fn return_(&mut self, offset: Expr, len: Expr) {
        self.statement.push(IR::Result { offset, len });
    }

    pub fn mstore(&mut self, addr: Expr, var: Expr) {
        self.statement.push(IR::MemStore { addr, var });
    }

    pub fn mstore8(&mut self, addr: Expr, var: Expr) {
        self.statement.push(IR::MemStore8 { addr, var });
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
