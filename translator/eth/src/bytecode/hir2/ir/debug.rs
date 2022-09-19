#![allow(dead_code)]

use crate::bytecode::hir2::ir::expression::Expr;
use crate::bytecode::hir2::ir::statement::Statement;
use crate::bytecode::hir2::ir::Hir2;
use anyhow::Error;
use std::fmt::Write;

pub fn print_ir(ir: &Hir2, name: &str) -> Result<(), Error> {
    let mut buf = String::new();
    writeln!(buf, "HIR2 for {}:", name)?;
    write_buf(ir, &mut buf, 0)?;
    writeln!(
        buf,
        "================================================================================="
    )?;
    println!("{}", buf);
    Ok(())
}

fn write_buf(ir: &Hir2, buf: &mut String, width: usize) -> Result<(), Error> {
    write_statements(&ir.statements, buf, width)?;
    Ok(())
}

fn write_statements(inst: &[Statement], buf: &mut String, width: usize) -> Result<(), Error> {
    for inst in inst {
        write_statement(inst, buf, width)?;
    }
    Ok(())
}

fn write_statement(inst: &Statement, buf: &mut String, width: usize) -> Result<(), Error> {
    match inst {
        Statement::Assign { var, expr } => {
            write!(buf, "{:width$}let var{:?} = ", " ", var.index())?;
            print_var_expression(expr, buf)?;
            writeln!(buf, ";")?;
        }
        Statement::MemStore8 { addr, var } => {
            write!(buf, "{:width$}mem8[", " ",)?;
            print_var_expression(addr, buf)?;
            write!(buf, "] = ")?;
            print_var_expression(var, buf)?;
            writeln!(buf, ";")?;
        }
        Statement::MemStore { addr, var } => {
            write!(buf, "{:width$}mem[", " ",)?;
            print_var_expression(addr, buf)?;
            write!(buf, "] = ")?;
            print_var_expression(var, buf)?;
            writeln!(buf, ";")?;
        }
        Statement::SStore { addr, var } => {
            write!(buf, "{:width$}sstore[", " ",)?;
            print_var_expression(addr, buf)?;
            write!(buf, "] = ")?;
            print_var_expression(var, buf)?;
            writeln!(buf, ";")?;
        }
        Statement::Log {
            offset,
            len,
            topics,
        } => {
            write!(buf, "{:width$}log[mem[", " ",)?;
            print_var_expression(offset, buf)?;
            write!(buf, ":+")?;
            print_var_expression(len, buf)?;
            write!(buf, "]](")?;
            for (i, topic) in topics.iter().enumerate() {
                if i > 0 {
                    write!(buf, ", ")?;
                }
                print_var_expression(topic, buf)?;
            }
            writeln!(buf, ");")?;
        }
        Statement::If {
            condition,
            true_branch,
            false_branch,
        } => {
            write!(buf, "{:width$}if ", " ",)?;
            print_var_expression(condition, buf)?;
            writeln!(buf, " {{")?;
            write_statements(true_branch, buf, width + 4)?;
            writeln!(buf, "{:width$}}} else {{", " ", width = width)?;
            write_statements(false_branch, buf, width + 4)?;
            writeln!(buf, "{:width$}}}", " ", width = width)?;
        }
        Statement::Loop {
            id,
            condition_block,
            condition,
            is_true_br_loop,
            loop_br,
        } => {
            writeln!(buf, "{:width$}'{}: loop {{", " ", id, width = width)?;
            write_statements(condition_block, buf, width + 4)?;
            write!(buf, "{:width$}if ", " ", width = width + 4)?;
            print_var_expression(condition, buf)?;
            writeln!(buf, " {{")?;
            write_statements(loop_br, buf, width + 8)?;
            writeln!(buf, "{:width$}}} else {{", " ", width = width + 4)?;
            if *is_true_br_loop {
                writeln!(buf, "{:width$}break {:?};", " ", id, width = width + 8)?;
            } else {
                write_statements(loop_br, buf, width + 8)?;
            }
            writeln!(buf, "{:width$}}}", " ", width = width + 4)?;
            writeln!(buf, "{:width$}}}", " ", width = width)?;
        }
        Statement::Continue { loop_id, context } => {
            write!(buf, "{:width$}{{", " ")?;
            write_statements(context, buf, width + 4)?;
            writeln!(buf, "{:width$}}}", " ")?;
            writeln!(buf, "{:width$}continue {:?};", " ", loop_id)?;
        }
        Statement::Stop => {
            writeln!(buf, "{:width$}stop;", " ")?;
        }
        Statement::Abort(code) => {
            writeln!(buf, "{:width$}abort({});", " ", code)?;
        }
        Statement::Result { offset, len } => {
            write!(buf, "{:width$}return mem[", " ",)?;
            print_var_expression(offset, buf)?;
            write!(buf, ":+")?;
            print_var_expression(len, buf)?;
            writeln!(buf, "];")?;
        }
        Statement::Move(val) => {
            write!(buf, "{:width$}move(", " ",)?;
            print_var_expression(val, buf)?;
            writeln!(buf, ")")?;
        }
    }
    Ok(())
}

fn print_var_expression(expr: &Expr, buf: &mut String) -> Result<(), Error> {
    match expr {
        Expr::Val(val) => {
            write!(buf, "{:?}", val)?;
        }
        Expr::Var(var) => {
            write!(buf, "var{:?}", var.index())?;
        }
        Expr::MLoad { mem_offset } => {
            write!(buf, "mem[")?;
            print_var_expression(mem_offset, buf)?;
            write!(buf, "]")?;
        }
        Expr::SLoad { key } => {
            write!(buf, "sload[")?;
            print_var_expression(key, buf)?;
            write!(buf, "]")?;
        }
        Expr::Signer => {
            write!(buf, "signer")?;
        }
        Expr::MSize => {
            write!(buf, "mem_size")?;
        }
        Expr::ArgsSize => {
            write!(buf, "args_size")?;
        }
        Expr::Args { args_offset } => {
            write!(buf, "args[")?;
            print_var_expression(args_offset, buf)?;
            write!(buf, "]")?;
        }
        Expr::UnaryOp(cmd, op) => {
            write!(buf, "({}", cmd)?;
            print_var_expression(op, buf)?;
            write!(buf, ")")?;
        }
        Expr::BinaryOp(cmd, op1, op2) => {
            write!(buf, "(")?;
            print_var_expression(op1, buf)?;
            write!(buf, " {} ", cmd)?;
            print_var_expression(op2, buf)?;
            write!(buf, ")")?;
        }
        Expr::TernaryOp(cmd, op, op1, op2) => {
            write!(buf, "(")?;
            print_var_expression(op, buf)?;
            write!(buf, " {} ", cmd)?;
            print_var_expression(op1, buf)?;
            write!(buf, " : ")?;
            print_var_expression(op2, buf)?;
            write!(buf, ")")?;
        }
        Expr::Hash {
            mem_offset,
            mem_len,
        } => {
            write!(buf, "hash[mem[")?;
            print_var_expression(mem_offset, buf)?;
            write!(buf, ":+")?;
            print_var_expression(mem_len, buf)?;
            write!(buf, "]]")?;
        }
    }
    Ok(())
}
