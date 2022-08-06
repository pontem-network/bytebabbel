use crate::bytecode::llir::executor::math::{BinaryOp, UnaryOp};
use crate::bytecode::llir::ir::instruction::Instruction;
use crate::bytecode::llir::ir::var::Var;
use crate::Ir;
use anyhow::Error;
use log::log_enabled;
use log::Level;
use std::fmt::Write;

pub fn print_ir(ir: &Ir) {
    if log_enabled!(Level::Trace) {
        let mut buf = String::new();
        if let Err(err) = print_buf(ir, &mut buf, 0) {
            log::error!("Failed to print ir: {}", err);
        }
        log::trace!("IR:\n{}", buf);
    }
}

fn print_buf(ir: &Ir, buf: &mut String, width: usize) -> Result<(), Error> {
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
    ir: &Ir,
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
    ir: &Ir,
    inst: &Instruction,
    buf: &mut String,
    width: usize,
) -> Result<(), Error> {
    match inst {
        Instruction::SetVar(id) => {
            write!(buf, "{:width$}let {:?} = ", " ", id)?;
            print_ir_var(&ir.var(id), buf, 0)?;
        }
        Instruction::MemStore(addr, val) => {
            let mut addr_bur = [0u8; 32];
            addr.to_big_endian(&mut addr_bur);
            write!(
                buf,
                "{:width$}let mem[{}] = ",
                " ",
                hex::encode(&mut addr_bur)
            )?;
            print_ir_var(&ir.var(val), buf, 0)?;
        }
        Instruction::Branch {
            condition,
            true_branch_len,
            false_branch_len,
        } => {
            write!(buf, "{:width$}if (", " ",)?;
            print_ir_var(&ir.var(condition), buf, 0)?;
            write!(buf, ") {{")?;
        }
    }
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
