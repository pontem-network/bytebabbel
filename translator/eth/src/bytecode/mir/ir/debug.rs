use std::fmt::{Display, Formatter, Write};

use crate::bytecode::hir::executor::math::TernaryOp;
use crate::bytecode::hir::ir::Label;
use crate::BlockId;
use anyhow::Error;
use log::log_enabled;
use log::Level;

use crate::bytecode::loc::Loc;
use crate::bytecode::mir::ir::expression::{Expression, TypedExpr};
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::Value;
use crate::bytecode::mir::ir::Mir;
use crate::bytecode::mir::translation::variables::Variable;

pub fn print_ir(ir: &Mir, name: &str) {
    if log_enabled!(Level::Trace) {
        let mut buf = String::new();
        buf.push_str("\nIR for ");
        buf.push_str(name);
        buf.push('\n');
        if let Err(err) = print_buf(ir, &mut buf) {
            log::error!("Failed to print mir: {}", err);
        }
        log::trace!("MIR:\n{}", buf);
    }
}

pub fn print_buf<B: Write>(ir: &Mir, buf: &mut B) -> Result<(), Error> {
    writeln!(
        buf,
        "================================================================================="
    )?;
    print_statements(ir.statements(), buf)?;
    writeln!(
        buf,
        "================================================================================="
    )?;
    Ok(())
}

fn print_statements<B: Write>(st: &[Loc<Statement>], buf: &mut B) -> Result<(), Error> {
    for inst in st {
        writeln!(buf, "{}: {}", BlockId::from(inst.start), inst)?;
    }
    Ok(())
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::InitStorage(var) => {
                write!(f, "InitStorage({})", var)
            }
            Statement::StoreContext(ctx) => {
                writeln!(f, "[")?;
                for (var, loc) in ctx {
                    writeln!(f, "  {}: {};", var, loc)?;
                }
                write!(f, "]")
            }
            Statement::Assign(var, expr) => {
                write!(f, "{} = {};", var, expr)
            }
            Statement::MStore {
                memory,
                offset,
                val,
            } => {
                write!(f, "{}.MStore({}, {});", memory, offset, val)
            }
            Statement::MStore8 {
                memory,
                offset,
                val,
            } => {
                write!(f, "{}.MStore8({}, {});", memory, offset, val)
            }
            Statement::SStore { storage, key, val } => {
                write!(f, "{}.SStore({}, {});", storage, key, val)
            }
            Statement::Abort(code) => {
                write!(f, "Abort({});", code)
            }
            Statement::Result(vars) => {
                write!(f, "Result(")?;
                for (i, var) in vars.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", var)?;
                }
                write!(f, ");")
            }
            Statement::Log {
                storage,
                memory,
                offset,
                len,
                topics,
            } => {
                write!(f, "{}.Log({}, {}, {}, [", storage, memory, offset, len)?;
                for (i, topic) in topics.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", topic)?;
                }
                write!(f, "]);")
            }
            Statement::Label(l) => {
                write!(f, "'{}:", l)
            }
            Statement::BrTrue(cnd, l) => {
                write!(f, "if {} goto '{};", cnd, l)
            }
            Statement::Br(l) => {
                write!(f, "goto '{};", l)
            }
        }
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to)
    }
}

impl Display for TypedExpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.expr.as_ref() {
            Expression::Const(val) => {
                write!(f, "{}", val)
            }
            Expression::GetMem => {
                write!(f, "GetMem()")
            }
            Expression::GetStore => {
                write!(f, "GetStore()")
            }
            Expression::MLoad { memory, offset } => {
                write!(f, "{}.MLoad({})", memory, offset)
            }
            Expression::MSlice {
                memory,
                offset,
                len,
            } => {
                write!(f, "{}.MSlice({}, {})", memory, offset, len)
            }
            Expression::SLoad { storage, key } => {
                write!(f, "{}.SLoad({})", storage, key)
            }
            Expression::MSize { memory } => {
                write!(f, "{}.MSize()", memory)
            }
            Expression::MoveVar(var) => {
                write!(f, "move({})", var)
            }
            Expression::CopyVar(var) => {
                write!(f, "copy({})", var)
            }
            Expression::Unary(op, arg) => {
                write!(f, "{}({})", op, arg)
            }
            Expression::Binary(op, arg, arg1) => {
                write!(f, "({} {} {})", arg, op, arg1)
            }
            Expression::Ternary(op, arg1, arg2, arg3) => match op {
                TernaryOp::AddMod => {
                    write!(f, "addmod({}, {}, {})", arg1, arg2, arg3)
                }
                TernaryOp::MulMod => {
                    write!(f, "mulmod({}, {}, {})", arg1, arg2, arg3)
                }
            },
            Expression::Cast(expr, cast) => {
                write!(f, "({} as {})", expr, cast.to())
            }
            Expression::BytesLen(var) => {
                write!(f, "{}.BytesLen()", var)
            }
            Expression::ReadNum { data, offset } => {
                write!(f, "{}.ReadNum({})", data, offset)
            }
            Expression::Hash { mem, offset, len } => {
                write!(f, "{}.Hash({}, {})", mem, offset, len)
            }
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(v) => {
                write!(f, "{}", v)
            }
            Value::Bool(v) => {
                write!(f, "{}", v)
            }
        }
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "var_{}", self.index())
    }
}
