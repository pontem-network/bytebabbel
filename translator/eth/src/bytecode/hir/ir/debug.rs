use std::fmt::Write;

use anyhow::Error;
use log::log_enabled;
use log::Level;

use crate::bytecode::hir::executor::math::{BinaryOp, UnaryOp};
use crate::bytecode::hir::ir::statement::Statement;
use crate::bytecode::hir::ir::var::Expr;
use crate::Hir;

pub fn print_ir(ir: &Hir, name: &str) {
    if log_enabled!(Level::Trace) {
        let mut buf = String::new();
        buf.push_str("HIR for ");
        buf.push_str(name);
        buf.push_str(":\n");
        if let Err(err) = print_buf(ir, &mut buf, 0) {
            log::error!("Failed to print ir: {}", err);
        }
        log::trace!("IR:\n{}", buf);
    }
}

fn print_buf(ir: &Hir, buf: &mut String, width: usize) -> Result<(), Error> {
    writeln!(
        buf,
        "================================================================================="
    )?;
    print_instructions(ir, &ir.instructions, buf, width)?;
    writeln!(
        buf,
        "================================================================================="
    )?;
    Ok(())
}

fn print_instructions(
    ir: &Hir,
    inst: &[Statement],
    buf: &mut String,
    width: usize,
) -> Result<(), Error> {
    for inst in inst {
        print_instruction(ir, inst, buf, width)?;
    }
    Ok(())
}

fn print_instruction(
    ir: &Hir,
    inst: &Statement,
    buf: &mut String,
    width: usize,
) -> Result<(), Error> {
    match inst {
        Statement::SetVar(id) => {
            write!(buf, "{:width$}let {:?} = ", " ", id)?;
            print_ir_var(ir.var(id), buf, 0)?;
            write!(buf, ";")?;
        }
        Statement::If {
            condition,
            true_branch,
            false_branch,
        } => {
            writeln!(buf, "{:width$}if {:?} {{", " ", condition)?;
            print_instructions(ir, true_branch, buf, width + 4)?;
            writeln!(buf, "{:width$}}} else {{", " ",)?;
            print_instructions(ir, false_branch, buf, width + 4)?;
            write!(buf, "{:width$}}}", " ",)?;
        }
        Statement::Stop => {
            write!(buf, "{:width$}stop!;", " ")?;
        }
        Statement::Abort(code) => {
            write!(buf, "{:width$}abort!({});", " ", code)?;
        }
        Statement::Result { offset, len } => {
            write!(buf, "{:width$}return [{:?}; {:?}];", " ", offset, len)?;
        }
        Statement::Loop {
            id,
            condition_block,
            condition,
            is_true_br_loop,
            loop_br,
        } => {
            writeln!(buf, "{:width$}'{}: loop {{", " ", id)?;
            print_instructions(ir, condition_block, buf, width + 4)?;
            writeln!(
                buf,
                "{:width$}if {condition:?} {{",
                " ",
                width = width + 8,
                condition = condition
            )?;
            if *is_true_br_loop {
                print_instructions(ir, loop_br, buf, width + 12)?;
                writeln!(buf, "{:width$}}} else {{", " ", width = width + 8)?;
                writeln!(buf, "{:width$}break;", " ", width = width + 12)?;
                writeln!(buf, "{:width$}}}", " ", width = width + 8)?;
            } else {
                writeln!(buf, "{:width$}break;", " ", width = width + 12)?;
                writeln!(buf, "{:width$}}} else {{", " ", width = width + 8)?;
                print_instructions(ir, loop_br, buf, width + 12)?;
                writeln!(buf, "{:width$}}}", " ", width = width + 8)?;
            }
            write!(buf, "{:width$}}}", " ", width = width + 4)?;
        }
        Statement::Continue { loop_id, context } => {
            writeln!(buf, "{:width$}{{", " ",)?;
            print_instructions(ir, context, buf, width + 4)?;
            writeln!(buf, "{:width$}}}", " ",)?;
            write!(buf, "{:width$}condition {:?};", " ", loop_id)?;
        }
        Statement::MapVar { id, val } => {
            write!(buf, "{:width$}{:?} = {:?};", " ", id, val)?;
        }
        Statement::MemStore8 { addr, var } => {
            write!(buf, "{:width$}mem[{:?}] = {:?});", " ", addr, var)?;
        }
        Statement::MemStore { addr, var } => {
            write!(buf, "{:width$}mem[{:?}] = {:?};", " ", addr, var)?;
        }
        Statement::SStore { addr, var } => {
            write!(buf, "{:width$}store[{:?}] = {:?};", " ", addr, var)?;
        }
        Statement::Log {
            offset,
            len,
            topics,
        } => {
            write!(buf, "{:width$}log(mem[{:?}; {:?}], ", " ", offset, len)?;
            for topic in topics {
                write!(buf, "{:?}, ", topic)?;
            }
            write!(buf, ");")?;
        }
    };
    writeln!(buf)?;
    Ok(())
}

fn print_ir_var(var: &Expr, buf: &mut String, width: usize) -> Result<(), Error> {
    match var {
        Expr::Val(val) => {
            write!(buf, "{:width$}{:?}", " ", val)?;
        }
        Expr::UnaryOp(cmd, op1) => {
            match cmd {
                UnaryOp::IsZero => write!(buf, "{:width$}{:?} == 0", " ", op1)?,
                UnaryOp::Not => write!(buf, "{:width$}!{:?}", " ", op1)?,
            };
        }
        Expr::BinaryOp(cmd, op1, op2) => {
            write!(buf, "{:width$}{:?} {} {:?}", " ", op1, cmd.sign(), op2)?;
        }
        Expr::TernaryOp(cmd, op1, op2, op3) => {
            write!(
                buf,
                "{:width$}{:?}({:?}, {:?}, {:?})",
                " ", cmd, op1, op2, op3
            )?;
        }
        Expr::MLoad(addr) => {
            write!(buf, "{:width$}mem[{:?}]", " ", addr)?;
        }
        Expr::SLoad(addr) => {
            write!(buf, "{:width$}store[{:?}]", " ", addr)?;
        }
        Expr::MSize => {
            write!(buf, "{:width$}mem.len()", " ",)?;
        }
        Expr::Signer => {
            write!(buf, "{:width$}signer", " ",)?;
        }
        Expr::ArgsSize => {
            write!(buf, "{:width$}args.len()", " ",)?;
        }
        Expr::Args(offset) => {
            write!(buf, "{:width$}args[{:?}]", " ", offset)?;
        }
        Expr::Hash(offset, len) => {
            write!(buf, "{:width$}hash(memory({:?}, {:?}))", " ", offset, len)?;
        }
        Expr::Var(var) => {
            write!(buf, "{:width$}var_{:?}", " ", var)?;
        }
    }
    Ok(())
}

impl BinaryOp {
    pub fn sign(&self) -> &str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Lt => "<",
            BinaryOp::Gt => ">",
            BinaryOp::Eq => "==",
            BinaryOp::Shr => ">>",
            BinaryOp::Shl => "<<",
            BinaryOp::Sar => ">!>",
            BinaryOp::And => "&",
            BinaryOp::Or => "|",
            BinaryOp::Xor => "^",
            BinaryOp::SDiv => "//",
            BinaryOp::SLt => "<!",
            BinaryOp::SGt => ">!",
            BinaryOp::Byte => "byte",
            BinaryOp::SMod => "%!",
            BinaryOp::Exp => "**",
            BinaryOp::SignExtend => "**!",
        }
    }
}
