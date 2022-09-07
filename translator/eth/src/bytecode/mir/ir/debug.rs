use crate::bytecode::mir::ir::expression::{Expression, StackOp};
use crate::bytecode::mir::ir::math::Operation;
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
        Statement::CreateVar(var, value) => {
            write!(
                buf,
                "{:width$}let var_{:?}: {} = ",
                " ",
                var.index(),
                var.s_type()
            )?;
            print_expr(value, buf, width + 4)?;
            writeln!(buf, ";")?;
        }
        Statement::IF {
            cnd,
            true_br,
            false_br,
        } => {
            write!(buf, "{:width$}if ", " ")?;
            print_expr(cnd, buf, width + 4)?;
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
            write!(buf, "{:width$}if ", " ", width = width + 4)?;
            print_expr(cnd, buf, width)?;
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
                write!(buf, "var_{:?}", value.index())?;
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
                "{:width$}{:?}.mem_store(var_{:?}, var_{:?});",
                " ",
                memory.index(),
                offset.index(),
                val.index()
            )?;
        }
        Statement::MStore8 {
            memory,
            offset,
            val,
        } => {
            writeln!(
                buf,
                "{:width$}{:?}.mem_store8(var_{:?}, var_{:?});",
                " ",
                memory.index(),
                offset.index(),
                val.index()
            )?;
        }
        Statement::SStore {
            storage,
            offset,
            val,
        } => {
            writeln!(
                buf,
                "{:width$}{:?}.state_store(var_{:?}, var_{:?});",
                " ",
                storage.index(),
                offset.index(),
                val.index()
            )?;
        }
        Statement::InitStorage(var) => {
            writeln!(buf, "{:width$}init_storage(var_{:?});", " ", var.index(),)?;
        }
    }
    Ok(())
}

pub fn print_expr(expr: &Expression, buf: &mut String, width: usize) -> Result<(), Error> {
    match expr {
        Expression::Const(val) => match val {
            Value::U128(val) => write!(buf, "{}u128", val)?,
            Value::Bool(val) => write!(buf, "{}", val)?,
        },
        Expression::Var(val) => write!(buf, "var_{}", val.index())?,
        Expression::Param(idx, _) => write!(buf, "param_{}", idx)?,
        Expression::Operation(cmd, op, op1) => {
            write!(
                buf,
                "(var_{} {} var_{})",
                op.index(),
                cmd.sign(),
                op1.index()
            )?;
        }
        Expression::StackOps(ops) => {
            writeln!(buf, "{{")?;
            for op in &ops.vec {
                print_stack_op(op, buf, width + 2)?;
                writeln!(buf, ";")?;
            }
            write!(buf, "{:width$}}}", " ")?;
        }
        Expression::MLoad { memory, offset } => {
            write!(
                buf,
                "var_{:?}.mem_load(var_{:?})",
                memory.index(),
                offset.index()
            )?;
        }
        Expression::SLoad { storage, offset } => {
            write!(buf, "var_{:?}.state_load(var_{:?})", storage, offset)?;
        }
        Expression::MSize { memory } => {
            write!(buf, "var_{:?}.mem_len()", memory)?;
        }
        Expression::GetMem => {
            write!(buf, "contract_memory()")?;
        }
        Expression::GetStore => {
            write!(buf, "borrow_storage()")?;
        }
    }
    Ok(())
}

fn print_stack_op(op: &StackOp, buf: &mut String, width: usize) -> Result<(), Error> {
    match op {
        StackOp::PushConst(val) => match val {
            Value::U128(val) => {
                write!(buf, "{:width$}push {:?}", " ", val)?;
            }
            Value::Bool(val) => {
                write!(buf, "{:width$}push {:?}", " ", val)?;
            }
        },
        StackOp::PushVar(val) => {
            write!(buf, "{:width$}push var_{}", " ", val.index())?;
        }
        StackOp::BinaryOp(val) => {
            write!(buf, "{:width$}{:?}", " ", val)?;
        }
        StackOp::Not => {
            write!(buf, "{:width$}!", " ")?;
        }
    }
    Ok(())
}

impl Operation {
    pub fn sign(&self) -> &str {
        match self {
            Operation::Add => "+",
            Operation::Sub => "-",
            Operation::Mul => "*",
            Operation::Div => "/",
            Operation::Mod => "%",
            Operation::Lt => "<",
            Operation::Gt => ">",
            Operation::Shr => ">>",
            Operation::Shl => "<<",
            Operation::And => "&",
            Operation::Or => "|",
            Operation::Xor => "^",
            Operation::BitOr => "|",
            Operation::BitAnd => "&",
            Operation::Not => "!",
            Operation::Eq => "==",
            Operation::Neq => "!=",
            Operation::Le => "<=",
            Operation::Ge => ">=",
        }
    }
}
