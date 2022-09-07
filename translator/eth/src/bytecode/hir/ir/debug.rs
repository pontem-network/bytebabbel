use crate::bytecode::hir::executor::math::{BinaryOp, UnaryOp};
use crate::bytecode::hir::ir::instruction::Instruction;
use crate::bytecode::hir::ir::var::Var;
use crate::Hir;
use anyhow::Error;
use log::log_enabled;
use log::Level;
use std::fmt::Write;

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
    inst: &[Instruction],
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
    inst: &Instruction,
    buf: &mut String,
    width: usize,
) -> Result<(), Error> {
    match inst {
        Instruction::SetVar(id) => {
            write!(buf, "{:width$}let {:?} = ", " ", id)?;
            print_ir_var(ir.var(id), buf, 0)?;
            write!(buf, ";")?;
        }
        Instruction::If {
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
        Instruction::Stop => {
            write!(buf, "{:width$}stop!;", " ")?;
        }
        Instruction::Abort(code) => {
            write!(buf, "{:width$}abort!({});", " ", code)?;
        }
        Instruction::Result { offset, len } => {
            write!(buf, "{:width$}return [{:?}; {:?}];", " ", offset, len)?;
        }
        Instruction::Loop {
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
        Instruction::Continue { loop_id, context } => {
            writeln!(buf, "{:width$}{{", " ",)?;
            print_instructions(ir, context, buf, width + 4)?;
            writeln!(buf, "{:width$}}}", " ",)?;
            write!(buf, "{:width$}condition {:?};", " ", loop_id)?;
        }
        Instruction::MapVar { id, val } => {
            write!(buf, "{:width$}{:?} = {:?};", " ", id, val)?;
        }
        Instruction::MemStore8 { addr, var } => {
            write!(buf, "{:width$}mem[{:?}] = {:?});", " ", addr, var)?;
        }
        Instruction::MemStore { addr, var } => {
            write!(buf, "{:width$}mem[{:?}] = {:?};", " ", addr, var)?;
        }
        Instruction::SStore { addr, var } => {
            write!(buf, "{:width$}store[{:?}] = {:?};", " ", addr, var)?;
        }
    };
    writeln!(buf)?;
    Ok(())
}

fn print_ir_var(var: &Var, buf: &mut String, width: usize) -> Result<(), Error> {
    match var {
        Var::Val(val) => {
            write!(buf, "{:width$}{:?}", " ", val)?;
        }
        Var::Param(param) => {
            write!(buf, "{:width$}param_{:?}", " ", param)?;
        }
        Var::UnaryOp(cmd, op1) => {
            match cmd {
                UnaryOp::IsZero => write!(buf, "{:width$}{:?} == 0", " ", op1)?,
                UnaryOp::Not => write!(buf, "{:width$}!{:?}", " ", op1)?,
            };
        }
        Var::BinaryOp(cmd, op1, op2) => {
            write!(buf, "{:width$}{:?} {} {:?}", " ", op1, cmd.sign(), op2)?;
        }
        Var::TernaryOp(cmd, op1, op2, op3) => {
            write!(
                buf,
                "{:width$}{:?}({:?}, {:?}, {:?})",
                " ", cmd, op1, op2, op3
            )?;
        }
        Var::MLoad(addr) => {
            write!(buf, "{:width$}mem[{:?}]", " ", addr)?;
        }
        Var::SLoad(addr) => {
            write!(buf, "{:width$}store[{:?}]", " ", addr)?;
        }
        Var::MSize => {
            write!(buf, "{:width$}mem.len()", " ",)?;
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
            BinaryOp::EQ => "==",
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
