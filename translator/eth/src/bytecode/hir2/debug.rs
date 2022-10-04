use crate::bytecode::hir2::executor::math::{BinaryOp, TernaryOp, UnaryOp};
use crate::bytecode::hir2::ir::Expr;
use anyhow::Error;
use std::fmt::{Display, Formatter, Write};

pub fn print_expr<B: Write>(buf: &mut B, expr: &Expr) -> Result<(), Error> {
    match expr {
        Expr::Val(val) => write!(buf, "{}", val)?,
        Expr::Var(var) => write!(buf, "{}", var)?,
        Expr::MLoad(offset) => {
            write!(buf, "mload(")?;
            print_expr(buf, &offset)?;
            write!(buf, ")")?
        }
        Expr::SLoad(key) => {
            write!(buf, "sload(")?;
            print_expr(buf, &key)?;
            write!(buf, ")")?
        }
        Expr::Signer => write!(buf, "signer")?,
        Expr::MSize => write!(buf, "msize")?,
        Expr::ArgsSize => write!(buf, "args_size")?,
        Expr::Args(idx) => {
            write!(buf, "args(")?;
            print_expr(buf, &idx)?;
            write!(buf, ")")?
        }
        Expr::UnaryOp(cmd, arg) => {
            write!(buf, "{}", cmd)?;
            print_expr(buf, &arg)?;
        }
        Expr::BinaryOp(cmd, arg1, arg2) => {
            print_expr(buf, &arg1)?;
            write!(buf, " {} ", cmd)?;
            print_expr(buf, &arg2)?;
        }
        Expr::TernaryOp(cmd, arg1, arg2, arg3) => match cmd {
            TernaryOp::AddMod => {
                write!(buf, "(")?;
                print_expr(buf, &arg1)?;
                write!(buf, " + ")?;
                print_expr(buf, &arg2)?;
                write!(buf, ") % ")?;
                print_expr(buf, &arg3)?;
            }
            TernaryOp::MulMod => {
                write!(buf, "(")?;
                print_expr(buf, &arg1)?;
                write!(buf, " * ")?;
                print_expr(buf, &arg2)?;
                write!(buf, ") % ")?;
                print_expr(buf, &arg3)?;
            }
        },
        Expr::Hash(offset, len) => {
            write!(buf, "hash(")?;
            print_expr(buf, &offset)?;
            write!(buf, ", ")?;
            print_expr(buf, &len)?;
            write!(buf, ")")?
        }
        Expr::Copy(cp) => {
            write!(buf, "copy(")?;
            print_expr(buf, &cp)?;
            write!(buf, ")")?
        }
    }
    Ok(())
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOp::IsZero => write!(f, "0 == "),
            UnaryOp::Not => write!(f, "!"),
        }
    }
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOp::Eq => write!(f, " == "),
            BinaryOp::Lt => write!(f, " < "),
            BinaryOp::Gt => write!(f, " > "),
            BinaryOp::Shr => write!(f, " >> "),
            BinaryOp::Shl => write!(f, " << "),
            BinaryOp::Sar => write!(f, " >>> "),
            BinaryOp::Add => write!(f, " + "),
            BinaryOp::And => write!(f, " & "),
            BinaryOp::Or => write!(f, " | "),
            BinaryOp::Xor => write!(f, " ^ "),
            BinaryOp::Mul => write!(f, " * "),
            BinaryOp::Sub => write!(f, " - "),
            BinaryOp::Div => write!(f, " / "),
            BinaryOp::SDiv => write!(f, " // "),
            BinaryOp::SLt => write!(f, " < "),
            BinaryOp::SGt => write!(f, " > "),
            BinaryOp::Byte => write!(f, "#"),
            BinaryOp::Mod => write!(f, " % "),
            BinaryOp::SMod => write!(f, " %% "),
            BinaryOp::Exp => write!(f, " ** "),
            BinaryOp::SignExtend => write!(f, " ** "),
        }
    }
}
