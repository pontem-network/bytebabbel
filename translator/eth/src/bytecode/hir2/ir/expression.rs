use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::executor::math::{BinaryOp, TernaryOp, UnaryOp};
use crate::bytecode::hir2::vars::VarId;
use primitive_types::U256;
use std::rc::Rc;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub enum Expr {
    Val(U256),
    Var(VarId),
    MLoad {
        mem_offset: Rc<Expr>,
    },
    SLoad {
        key: Rc<Expr>,
    },
    Signer,
    MSize,
    ArgsSize,
    Args {
        args_offset: Rc<Expr>,
    },
    UnaryOp(UnaryOp, Rc<Expr>),
    BinaryOp(BinaryOp, Rc<Expr>, Rc<Expr>),
    TernaryOp(TernaryOp, Rc<Expr>, Rc<Expr>, Rc<Expr>),
    Hash {
        mem_offset: Rc<Expr>,
        mem_len: Rc<Expr>,
    },
}

impl Expr {
    pub fn resolve(&self, ctx: &mut Context) -> Option<U256> {
        match self {
            Expr::Val(val) => Some(*val),
            Expr::Var(var) => {
                ctx.const_pool().use_var(var);
                ctx.vars().get(var)?.0.resolve(ctx)
            }
            Expr::MLoad { .. } => None,
            Expr::SLoad { .. } => None,
            Expr::Signer => None,
            Expr::MSize => None,
            Expr::ArgsSize => {
                if ctx.is_static_analysis_enable() {
                    Some(ctx.fun().call_data_size())
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
                let op1 = op1.resolve(ctx)?;
                let op2 = op2.resolve(ctx)?;
                Some(cmd.calc(op1, op2))
            }
            Expr::TernaryOp(cmd, op1, op2, op3) => {
                let op1 = op1.resolve(ctx)?;
                let op2 = op2.resolve(ctx)?;
                let op3 = op3.resolve(ctx)?;
                Some(cmd.calc(op1, op2, op3))
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
