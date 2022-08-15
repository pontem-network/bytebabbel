use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::Mir;
use anyhow::Error;
use log::log_enabled;
use log::Level;
use std::fmt::Write;

pub fn print_ir(ir: &Mir) {
    if log_enabled!(Level::Trace) {
        let mut buf = String::new();
        if let Err(err) = print_buf(ir, &mut buf, 0) {
            log::error!("Failed to print mir: {}", err);
        }
        log::trace!("MIR:\n{}", buf);
    }
}

fn print_buf(ir: &Mir, buf: &mut String, width: usize) -> Result<(), Error> {
    writeln!(
        buf,
        "================================================================================="
    )?;
    print_statements(&ir.as_statements(), buf, width)?;
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
    writeln!(buf)?;
    Ok(())
}

// fn print_ir_var(var: &Var, buf: &mut String, width: usize) -> Result<(), Error> {
//     match var {
//         Var::Val(val) => {
//             write!(buf, "{:width$}{:?}", " ", val)?;
//         }
//         Var::Param(param) => {
//             write!(buf, "{:width$}param_{:?}", " ", param)?;
//         }
//         Var::UnaryOp(cmd, op1) => {
//             match cmd {
//                 UnaryOp::IsZero => write!(buf, "{:width$}{:?} == 0", " ", op1)?,
//                 UnaryOp::Not => write!(buf, "{:width$}!{:?}", " ", op1)?,
//             };
//         }
//         Var::BinaryOp(cmd, op1, op2) => {
//             write!(buf, "{:width$}{:?} {} {:?}", " ", op1, cmd.sign(), op2)?;
//         }
//         Var::TernaryOp(cmd, op1, op2, op3) => {
//             write!(
//                 buf,
//                 "{:width$}{:?}({:?}, {:?}, {:?})",
//                 " ", cmd, op1, op2, op3
//             )?;
//         }
//     }
//     Ok(())
// }
//
// impl BinaryOp {
//     pub fn sign(&self) -> &str {
//         match self {
//             BinaryOp::Add => "+",
//             BinaryOp::Sub => "-",
//             BinaryOp::Mul => "*",
//             BinaryOp::Div => "/",
//             BinaryOp::Mod => "%",
//             BinaryOp::Lt => "<",
//             BinaryOp::Gt => ">",
//             BinaryOp::EQ => "==",
//             BinaryOp::Shr => ">>",
//             BinaryOp::Shl => "<<",
//             BinaryOp::Sar => ">!>",
//             BinaryOp::And => "&",
//             BinaryOp::Or => "|",
//             BinaryOp::Xor => "^",
//             BinaryOp::SDiv => "//",
//             BinaryOp::SLt => "<!",
//             BinaryOp::SGt => ">!",
//             BinaryOp::Byte => "byte",
//             BinaryOp::SMod => "%!",
//             BinaryOp::Exp => "**",
//             BinaryOp::SignExtend => "**!",
//         }
//     }
// }
