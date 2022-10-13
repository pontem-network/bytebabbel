use std::fmt::{Display, Formatter, Write};

use crate::bytecode::hir::executor::math::TernaryOp;

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
        writeln!(buf, "{}: {}", inst.start, inst.as_ref())?;
    }
    Ok(())
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::InitStorage(var) => {
                write!(f, "InitStorage({})", var)
            }
            Statement::StoreStack(ctx) => {
                writeln!(f, "[")?;
                for (var, loc) in ctx {
                    writeln!(f, "  {}: {};", var, loc.as_ref())?;
                }
                write!(f, "]")
            }
            Statement::Assign(var, expr) => {
                write!(f, "{} = {};", var, expr.as_ref())
            }
            Statement::MStore {
                memory,
                offset,
                val,
            } => {
                write!(
                    f,
                    "{}.MStore({}, {});",
                    memory,
                    offset.as_ref(),
                    val.as_ref()
                )
            }
            Statement::MStore8 {
                memory,
                offset,
                val,
            } => {
                write!(
                    f,
                    "{}.MStore8({}, {});",
                    memory,
                    offset.as_ref(),
                    val.as_ref()
                )
            }
            Statement::SStore { storage, key, val } => {
                write!(f, "{}.SStore({}, {});", storage, key.as_ref(), val.as_ref())
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
                write!(
                    f,
                    "{}.Log({}, {}, {}, [",
                    storage,
                    memory,
                    offset.as_ref(),
                    len.as_ref()
                )?;
                for (i, topic) in topics.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", topic.as_ref())?;
                }
                write!(f, "]);")
            }
            Statement::Label(l) => {
                write!(f, "'{}:", l)
            }
            Statement::BrTrue(cnd, l) => {
                write!(f, "if {} goto '{};", cnd.as_ref(), l)
            }
            Statement::Br(l) => {
                write!(f, "goto '{};", l)
            }
        }
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
                write!(f, "{}.MLoad({})", memory, offset.as_ref())
            }
            Expression::MSlice {
                memory,
                offset,
                len,
            } => {
                write!(
                    f,
                    "{}.MSlice({}, {})",
                    memory,
                    offset.as_ref(),
                    len.as_ref()
                )
            }
            Expression::SLoad { storage, key } => {
                write!(f, "{}.SLoad({})", storage, key.as_ref())
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
                write!(f, "{}({})", op, arg.as_ref())
            }
            Expression::Binary(op, arg, arg1) => {
                write!(f, "({} {} {})", arg.as_ref(), op, arg1.as_ref())
            }
            Expression::Ternary(op, arg1, arg2, arg3) => match op {
                TernaryOp::AddMod => {
                    write!(
                        f,
                        "addmod({}, {}, {})",
                        arg1.as_ref(),
                        arg2.as_ref(),
                        arg3.as_ref()
                    )
                }
                TernaryOp::MulMod => {
                    write!(
                        f,
                        "mulmod({}, {}, {})",
                        arg1.as_ref(),
                        arg2.as_ref(),
                        arg3.as_ref()
                    )
                }
            },
            Expression::Cast(expr, cast) => {
                write!(f, "({} as {})", expr.as_ref(), cast.to())
            }
            Expression::BytesLen(var) => {
                write!(f, "{}.BytesLen()", var)
            }
            Expression::ReadNum { data, offset } => {
                write!(f, "{}.ReadNum({})", data, offset.as_ref())
            }
            Expression::Hash { mem, offset, len } => {
                write!(f, "{}.Hash({}, {})", mem, offset.as_ref(), len.as_ref())
            }
            Expression::Balance { address } => {
                write!(f, "balance({address:?})")
            }
            Expression::Gas => {
                todo!()
            }
            Expression::GasPrice => {
                write!(f, "gas_price()")
            }
            Expression::GasLimit => {
                write!(f, "gas_limit()")
            }
            Expression::BlockTimestamp => {
                write!(f, "block_timestamp()")
            }
            Expression::BlockHeight => {
                write!(f, "block_height()")
            }
            Expression::BlockHash => {
                todo!()
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
        write!(f, "var_{}", self.index())
    }
}
