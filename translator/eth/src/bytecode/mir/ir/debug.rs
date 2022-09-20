use crate::bytecode::hir::executor::math::TernaryOp;
use crate::bytecode::mir::ir::expression::{Expression, StackOp, TypedExpr};
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::Value;
use crate::bytecode::mir::ir::Mir;
use anyhow::Error;
use log::log_enabled;
use log::Level;
use std::fmt::Write;

pub fn print_ir(ir: &Mir, name: &str) {
    if log_enabled!(Level::Trace) {
        let mut buf = String::new();
        buf.push_str("\nIR for ");
        buf.push_str(name);
        buf.push('\n');
        if let Err(err) = print_buf(ir, &mut buf, 0) {
            log::error!("Failed to print mir: {}", err);
        }
        log::trace!("MIR:\n{}", buf);
    }
}

pub fn print_buf(ir: &Mir, buf: &mut String, width: usize) -> Result<(), Error> {
    writeln!(
        buf,
        "================================================================================="
    )?;
    print_statements(ir.statements(), buf, width)?;
    writeln!(
        buf,
        "================================================================================="
    )?;
    Ok(())
}

fn print_statements(st: &[Statement], buf: &mut String, width: usize) -> Result<(), Error> {
    for inst in st {
        print_statement(inst, buf, width)?;
    }
    Ok(())
}

fn print_statement(inst: &Statement, buf: &mut String, width: usize) -> Result<(), Error> {
    match inst {
        Statement::Assign(var, value) => {
            writeln!(
                buf,
                "{:width$}let var_{:?}: {} = {};",
                " ",
                var.index(),
                var.s_type(),
                print_expr(value, 0)?
            )?;
        }
        Statement::IF {
            cnd,
            true_br,
            false_br,
        } => {
            write!(buf, "{:width$}if ({})", " ", print_expr(cnd, width)?)?;
            writeln!(buf, " {{")?;
            print_statements(true_br, buf, width + 4)?;
            writeln!(buf, "{:width$}}} else {{", " ",)?;
            print_statements(false_br, buf, width + 4)?;
            writeln!(buf, "{:width$}}}", " ")?;
        }
        Statement::Loop {
            id,
            cnd_calc,
            cnd,
            body,
        } => {
            writeln!(buf, "{:width$}'l{}: loop {{", " ", id)?;
            print_statements(cnd_calc, buf, width + 4)?;
            write!(
                buf,
                "{:width$}if ({})",
                " ",
                print_expr(cnd, 0)?,
                width = width + 4
            )?;
            writeln!(buf, "{:width$}{{", " ",)?;
            writeln!(buf, "{:width$} break 'l{};", " ", id, width = width + 4)?;
            writeln!(buf, "{:width$}}} else {{", " ", width = width + 4)?;
            print_statements(body, buf, width + 4)?;
            writeln!(buf, "{:width$}}}", " ",)?;
            writeln!(buf, "{:width$}}}", " ",)?;
        }
        Statement::Abort(code) => {
            writeln!(buf, "{:width$}abort({:?});", " ", code)?;
        }
        Statement::Result(ret) => {
            write!(buf, "{:width$}return (", " ")?;
            for (i, value) in ret.iter().enumerate() {
                if i > 0 {
                    write!(buf, ", ")?;
                }
                write!(buf, "{:?}", print_expr(value, 0)?)?;
            }
            writeln!(buf, ");")?;
        }
        Statement::Continue(id) => {
            writeln!(buf, "{:width$}continue 'l{:?};", " ", id)?;
        }
        Statement::MStore {
            memory,
            offset,
            val,
        } => {
            writeln!(
                buf,
                "{:width$}{:?}.mem_store({}, {});",
                " ",
                memory.index(),
                print_expr(offset, 0)?,
                print_expr(val, 0)?,
            )?;
        }
        Statement::MStore8 {
            memory,
            offset,
            val,
        } => {
            writeln!(
                buf,
                "{:width$}{:?}.mem_store8({}, {});",
                " ",
                memory.index(),
                print_expr(offset, 0)?,
                print_expr(val, 0)?,
            )?;
        }
        Statement::SStore { storage, key, val } => {
            writeln!(
                buf,
                "{:width$}{:?}.state_store(var_{:?}, var_{:?});",
                " ",
                storage.index(),
                print_expr(key, 0)?,
                print_expr(val, 0)?,
            )?;
        }
        Statement::InitStorage(var) => {
            writeln!(buf, "{:width$}init_storage(var_{:?});", " ", var.index(),)?;
        }
        Statement::Log {
            storage: _,
            memory: _,
            offset,
            len,
            topics,
        } => {
            let topics = topics
                .iter()
                .map(|t| print_expr(t, 0).unwrap_or_default())
                .collect::<Vec<_>>()
                .join(", ");
            writeln!(
                buf,
                "{:width$}log(mem[{}:+{}], {})",
                " ",
                print_expr(offset, 0)?,
                print_expr(len, 0)?,
                topics
            )?;
        }
    }
    Ok(())
}

pub fn print_expr(expr: &TypedExpr, width: usize) -> Result<String, Error> {
    let mut buf = String::new();
    match expr.expr.as_ref() {
        Expression::Const(val) => match val {
            Value::Number(val) => write!(buf, "{}", val)?,
            Value::Bool(val) => write!(buf, "{}", val)?,
        },
        Expression::Var(val) => write!(buf, "var_{}", val.index())?,
        Expression::StackOps(ops) => {
            writeln!(buf, "{{")?;
            for op in &ops.vec {
                print_stack_op(op, &mut buf, width + 2)?;
                writeln!(buf, ";")?;
            }
            write!(buf, "{:width$}}}", " ")?;
        }
        Expression::MLoad { memory, offset } => {
            write!(
                buf,
                "var_{:?}.mem_load({})",
                memory.index(),
                print_expr(offset, 0)?,
            )?;
        }
        Expression::SLoad {
            storage,
            key: offset,
        } => {
            write!(
                buf,
                "var_{:?}.state_load({})",
                storage.index(),
                print_expr(offset, 0)?,
            )?;
        }
        Expression::MSize { memory } => {
            write!(buf, "var_{:?}.mem_len()", memory.index())?;
        }
        Expression::GetMem => {
            write!(buf, "contract_memory()")?;
        }
        Expression::GetStore => {
            write!(buf, "borrow_storage()")?;
        }
        Expression::Cast(var, cast) => {
            write!(buf, "{} as {:?}", print_expr(var, 0)?, cast.to())?;
        }
        Expression::MSlice {
            memory,
            offset,
            len,
        } => {
            write!(
                buf,
                "var_{:?}.mem_slice({}, {})",
                memory.index(),
                print_expr(offset, 0)?,
                print_expr(len, 0)?,
            )?;
        }
        Expression::BytesLen(bytes) => {
            write!(buf, "var_{:?}.len()", bytes.index())?;
        }
        Expression::ReadNum { data, offset } => {
            write!(
                buf,
                "var_{:?}.read_num({:?})",
                data.index(),
                print_expr(offset, 0)?
            )?;
        }
        Expression::Hash { mem, offset, len } => {
            write!(
                buf,
                "var_{:?}.hash({}, {})",
                mem.index(),
                print_expr(offset, 0)?,
                print_expr(len, 0)?
            )?;
        }
        Expression::UnOp(cmd, op) => {
            write!(buf, "({}{})", cmd, print_expr(op, width)?)?;
        }
        Expression::BinOp(cmd, op, op1) => {
            write!(
                buf,
                "({} {:?} {})",
                print_expr(op, width)?,
                cmd,
                print_expr(op1, width)?
            )?;
        }
        Expression::TernOp(cmd, op, op1, op2) => match cmd {
            TernaryOp::AddMod => {
                write!(
                    buf,
                    "(({} + {}) % {})",
                    print_expr(op, width)?,
                    print_expr(op1, width)?,
                    print_expr(op2, width)?
                )?;
            }
            TernaryOp::MulMod => {
                write!(
                    buf,
                    "(({} * {}) % {})",
                    print_expr(op, width)?,
                    print_expr(op1, width)?,
                    print_expr(op2, width)?
                )?;
            }
        },
    }
    Ok(buf)
}

fn print_stack_op(op: &StackOp, buf: &mut String, width: usize) -> Result<(), Error> {
    match op {
        StackOp::Not => {
            write!(buf, "{:width$}!", " ")?;
        }
        StackOp::PushBool(val) => {
            write!(buf, "{:width$}push {}", " ", val)?;
        }
        StackOp::Eq => {
            write!(buf, "{:width$}eq", " ")?;
        }
        StackOp::PushBoolExpr(expr) => {
            write!(buf, "{:width$}push {}", " ", print_expr(expr, 0)?)?;
        }
    }
    Ok(())
}
