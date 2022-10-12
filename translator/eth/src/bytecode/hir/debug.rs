use crate::bytecode::hir::executor::math::{BinaryOp, TernaryOp, UnaryOp};
use crate::bytecode::hir::ir::{Stmt, _Expr};
use crate::bytecode::loc::Loc;
use crate::Offset;
use anyhow::Error;
use std::fmt::{Display, Formatter, Write};

pub fn print_expr<B: Write>(buf: &mut B, expr: &_Expr) -> Result<(), Error> {
    match expr {
        _Expr::Val(val) => write!(buf, "{}", val)?,
        _Expr::Var(var) => write!(buf, "{}", var)?,
        _Expr::MLoad(offset) => {
            write!(buf, "mload(")?;
            print_expr(buf, &offset)?;
            write!(buf, ")")?
        }
        _Expr::SLoad(key) => {
            write!(buf, "sload(")?;
            print_expr(buf, &key)?;
            write!(buf, ")")?
        }
        _Expr::Signer => write!(buf, "signer")?,
        _Expr::MSize => write!(buf, "msize")?,
        _Expr::ArgsSize => write!(buf, "args_size")?,
        _Expr::Args(idx) => {
            write!(buf, "args(")?;
            print_expr(buf, &idx)?;
            write!(buf, ")")?
        }
        _Expr::UnaryOp(cmd, arg) => {
            write!(buf, "(")?;
            write!(buf, "{}", cmd)?;
            print_expr(buf, &arg)?;
            write!(buf, ")")?
        }
        _Expr::BinaryOp(cmd, arg1, arg2) => {
            write!(buf, "(")?;
            print_expr(buf, &arg1)?;
            write!(buf, " {} ", cmd)?;
            print_expr(buf, &arg2)?;
            write!(buf, ")")?
        }
        _Expr::TernaryOp(cmd, arg1, arg2, arg3) => match cmd {
            TernaryOp::AddMod => {
                write!(buf, "((")?;
                print_expr(buf, &arg1)?;
                write!(buf, " + ")?;
                print_expr(buf, &arg2)?;
                write!(buf, ") % ")?;
                print_expr(buf, &arg3)?;
                write!(buf, ")")?
            }
            TernaryOp::MulMod => {
                write!(buf, "((")?;
                print_expr(buf, &arg1)?;
                write!(buf, " * ")?;
                print_expr(buf, &arg2)?;
                write!(buf, ") % ")?;
                print_expr(buf, &arg3)?;
                write!(buf, ")")?
            }
        },
        _Expr::Hash(offset, len) => {
            write!(buf, "hash(")?;
            print_expr(buf, &offset)?;
            write!(buf, ", ")?;
            print_expr(buf, &len)?;
            write!(buf, ")")?
        }
        _Expr::Copy(cp) => {
            write!(buf, "copy(")?;
            print_expr(buf, &cp)?;
            write!(buf, ")")?
        }
    }
    Ok(())
}

pub fn print_stmt<B: Write>(buf: &mut B, stmt: &Loc<Stmt>) -> Result<(), Error> {
    write!(buf, "{}: ", Offset::from(stmt.start))?;
    match stmt.as_ref() {
        Stmt::Label(label) => writeln!(buf, "'{}:", label)?,
        Stmt::Assign(var, expr) => {
            write!(buf, "{} = ", var)?;
            print_expr(buf, &expr)?;
            writeln!(buf, ";")?;
        }
        Stmt::MemStore8 { addr, val: var } => {
            write!(buf, "mstore8(")?;
            print_expr(buf, &addr)?;
            write!(buf, ", ")?;
            print_expr(buf, &var)?;
            writeln!(buf, ");")?;
        }
        Stmt::MemStore { addr, val: var } => {
            write!(buf, "mstore(")?;
            print_expr(buf, &addr)?;
            write!(buf, ", ")?;
            print_expr(buf, &var)?;
            writeln!(buf, ");")?;
        }
        Stmt::SStore {
            key: addr,
            val: var,
        } => {
            write!(buf, "sstore(")?;
            print_expr(buf, &addr)?;
            write!(buf, ", ")?;
            print_expr(buf, &var)?;
            writeln!(buf, ");")?;
        }
        Stmt::Log {
            offset,
            len,
            topics,
        } => {
            write!(buf, "log(")?;
            print_expr(buf, &offset)?;
            write!(buf, ", ")?;
            print_expr(buf, &len)?;
            for topic in topics {
                write!(buf, ", ")?;
                print_expr(buf, topic)?;
            }
            writeln!(buf, ");")?;
        }
        Stmt::Stop => {
            writeln!(buf, "stop;")?;
        }
        Stmt::Abort(code) => {
            writeln!(buf, "abort({});", code)?;
        }
        Stmt::Result { offset, len } => {
            write!(buf, "result(")?;
            print_expr(buf, &offset)?;
            write!(buf, ", ")?;
            print_expr(buf, &len)?;
            writeln!(buf, ");")?;
        }
        Stmt::Brunch(label) => {
            writeln!(buf, "goto {};", label)?;
        }
        Stmt::StoreStack(ctx) => {
            writeln!(buf, "[")?;
            for (id, expr) in ctx.iter() {
                write!(buf, "{}: ", Offset::from(expr.start))?;
                write!(buf, "{} = ", id)?;
                print_expr(buf, &expr)?;
                writeln!(buf, ";")?;
            }
            writeln!(buf, "]")?;
        }
        Stmt::BrunchTrue(cnd, true_br) => {
            print_expr(buf, &cnd)?;
            writeln!(buf, "\nBrTrue {};", true_br)?;
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
            BinaryOp::Eq => write!(f, "=="),
            BinaryOp::Lt => write!(f, "<"),
            BinaryOp::Gt => write!(f, ">"),
            BinaryOp::Shr => write!(f, ">>"),
            BinaryOp::Shl => write!(f, "<<"),
            BinaryOp::Sar => write!(f, ">>>"),
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::And => write!(f, "&"),
            BinaryOp::Or => write!(f, "|"),
            BinaryOp::Xor => write!(f, "^"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::Div => write!(f, "/"),
            BinaryOp::SDiv => write!(f, "//"),
            BinaryOp::SLt => write!(f, "<"),
            BinaryOp::SGt => write!(f, ">"),
            BinaryOp::Byte => write!(f, "#"),
            BinaryOp::Mod => write!(f, "%"),
            BinaryOp::SMod => write!(f, "%%"),
            BinaryOp::Exp => write!(f, "**"),
            BinaryOp::SignExtend => write!(f, "**"),
        }
    }
}
