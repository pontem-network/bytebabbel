use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::executor::math::{BinaryOp, TernaryOp, UnaryOp};
use crate::bytecode::hir::vars::VarId;
use primitive_types::U256;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum Expr {
    Val(U256),
    Var(VarId),
    MLoad {
        mem_offset: Box<Expr>,
    },
    SLoad {
        key: Box<Expr>,
    },
    Signer,
    MSize,
    ArgsSize,
    Args {
        args_offset: Box<Expr>,
    },
    UnaryOp(UnaryOp, Box<Expr>),
    BinaryOp(BinaryOp, Box<Expr>, Box<Expr>),
    TernaryOp(TernaryOp, Box<Expr>, Box<Expr>, Box<Expr>),
    Hash {
        mem_offset: Box<Expr>,
        mem_len: Box<Expr>,
    },
}

impl Expr {
    pub fn resolve(&self, ctx: Option<&mut Context>) -> Option<U256> {
        match self {
            Expr::Val(val) => Some(*val),
            Expr::Var(var) => {
                if let Some(ctx) = ctx {
                    ctx.const_pool().use_var(var);
                    ctx.vars().get(var)?.0.resolve(Some(ctx))
                } else {
                    None
                }
            }
            Expr::MLoad { .. } => None,
            Expr::SLoad { .. } => None,
            Expr::Signer => None,
            Expr::MSize => None,
            Expr::ArgsSize => {
                if let Some(ctx) = ctx {
                    if ctx.is_static_analysis_enable() {
                        Some(ctx.fun().call_data_size())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Expr::Args { .. } => None,
            Expr::UnaryOp(cmd, op) => {
                let val = op.resolve(ctx)?;
                Some(cmd.calc(val))
            }
            Expr::BinaryOp(cmd, op1, op2) => {
                if let Some(ctx) = ctx {
                    let op1 = op1.resolve(Some(ctx))?;
                    let op2 = op2.resolve(Some(ctx))?;
                    Some(cmd.calc(op1, op2))
                } else {
                    None
                }
            }
            Expr::TernaryOp(cmd, op1, op2, op3) => {
                if let Some(ctx) = ctx {
                    let op1 = op1.resolve(Some(ctx))?;
                    let op2 = op2.resolve(Some(ctx))?;
                    let op3 = op3.resolve(Some(ctx))?;
                    Some(cmd.calc(op1, op2, op3))
                } else {
                    None
                }
            }
            Expr::Hash { .. } => None,
        }
    }

    pub fn val(&self) -> Option<U256> {
        match self {
            Expr::Val(val) => Some(*val),
            _ => None,
        }
    }
}

impl From<U256> for Expr {
    fn from(val: U256) -> Self {
        Expr::Val(val)
    }
}
