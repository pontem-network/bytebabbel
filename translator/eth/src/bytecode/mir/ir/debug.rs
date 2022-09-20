use crate::bytecode::mir::ir::Mir;
use log::log_enabled;
use log::Level;

pub fn print_ir(_ir: &Mir, name: &str) {
    if log_enabled!(Level::Trace) {
        let mut buf = String::new();
        buf.push_str("\nIR for ");
        buf.push_str(name);
        buf.push('\n');
        // if let Err(err) = print_buf(ir, &mut buf, 0) {
        //     log::error!("Failed to print mir: {}", err);
        // }
        log::trace!("MIR:\n{}", buf);
    }
}
//
// pub fn print_buf(ir: &Mir, buf: &mut String, width: usize) -> Result<(), Error> {
//     writeln!(
//         buf,
//         "================================================================================="
//     )?;
//     print_statements(ir.statements(), buf, width)?;
//     writeln!(
//         buf,
//         "================================================================================="
//     )?;
//     Ok(())
// }
//
// fn print_statements(st: &[Statement], buf: &mut String, width: usize) -> Result<(), Error> {
//     for inst in st {
//         print_statement(inst, buf, width)?;
//     }
//     Ok(())
// }
//
// fn print_statement(inst: &Statement, buf: &mut String, width: usize) -> Result<(), Error> {
//     match inst {
//         Statement::Assign(var, value) => {
//             write!(
//                 buf,
//                 "{:width$}let var_{:?}: {} = ",
//                 " ",
//                 var.index(),
//                 var.s_type()
//             )?;
//             print_expr(value, buf, width + 4)?;
//             writeln!(buf, ";")?;
//         }
//         Statement::IF {
//             cnd,
//             true_br,
//             false_br,
//         } => {
//             write!(buf, "{:width$}if ", " ")?;
//             print_expr(cnd, buf, width + 4)?;
//             writeln!(buf, " {{")?;
//             print_statements(true_br, buf, width + 4)?;
//             writeln!(buf, "{:width$}}} else {{", " ",)?;
//             print_statements(false_br, buf, width + 4)?;
//             writeln!(buf, "{:width$}}}", " ")?;
//         }
//         Statement::Loop {
//             id,
//             cnd_calc,
//             cnd,
//             body,
//         } => {
//             writeln!(buf, "{:width$}'l{}: loop {{", " ", id)?;
//             print_statements(cnd_calc, buf, width + 4)?;
//             write!(buf, "{:width$}if ", " ", width = width + 4)?;
//             print_expr(cnd, buf, width)?;
//             writeln!(buf, "{:width$}{{", " ",)?;
//             writeln!(buf, "{:width$} break 'l{};", " ", id, width = width + 4)?;
//             writeln!(buf, "{:width$}}} else {{", " ", width = width + 4)?;
//             print_statements(body, buf, width + 4)?;
//             writeln!(buf, "{:width$}}}", " ",)?;
//             writeln!(buf, "{:width$}}}", " ",)?;
//         }
//         Statement::Abort(code) => {
//             writeln!(buf, "{:width$}abort({:?});", " ", code)?;
//         }
//         Statement::Result(ret) => {
//             write!(buf, "{:width$}return (", " ")?;
//             for (i, value) in ret.iter().enumerate() {
//                 if i > 0 {
//                     write!(buf, ", ")?;
//                 }
//                 write!(buf, "var_{:?}", value.index())?;
//             }
//             writeln!(buf, ");")?;
//         }
//         Statement::Continue(id) => {
//             writeln!(buf, "{:width$}continue 'l{:?};", " ", id)?;
//         }
//         Statement::MStore {
//             memory,
//             offset,
//             val,
//         } => {
//             writeln!(
//                 buf,
//                 "{:width$}{:?}.mem_store(var_{:?}, var_{:?});",
//                 " ",
//                 print_expr(memory, ),
//                 offset,
//                 val
//             )?;
//         }
//         Statement::MStore8 {
//             memory,
//             offset,
//             val,
//         } => {
//             writeln!(
//                 buf,
//                 "{:width$}{:?}.mem_store8(var_{:?}, var_{:?});",
//                 " ",
//                 memory.index(),
//                 offset.index(),
//                 val.index()
//             )?;
//         }
//         Statement::SStore {
//             storage,
//             key: offset,
//             val,
//         } => {
//             writeln!(
//                 buf,
//                 "{:width$}{:?}.state_store(var_{:?}, var_{:?});",
//                 " ",
//                 storage.index(),
//                 offset.index(),
//                 val.index()
//             )?;
//         }
//         Statement::InitStorage(var) => {
//             writeln!(buf, "{:width$}init_storage(var_{:?});", " ", var.index(),)?;
//         }
//         Statement::Log {
//             storage: _,
//             memory: _,
//             offset,
//             len,
//             topics,
//         } => {
//             let topics = topics
//                 .iter()
//                 .map(|t| format!("var_{:?}", t.index()))
//                 .collect::<Vec<_>>()
//                 .join(", ");
//             writeln!(
//                 buf,
//                 "{:width$}log(mem[{}:+{}], {})",
//                 " ",
//                 offset.index(),
//                 len.index(),
//                 topics
//             )?;
//         }
//     }
//     Ok(())
// }
//
// pub fn print_expr(expr: &TypedExpr, buf: &mut String, width: usize) -> Result<(), Error> {
//     match expr.expr.as_ref() {
//         Expression::Const(val) => match val {
//             Value::Number(val) => write!(buf, "{}", val)?,
//             Value::Bool(val) => write!(buf, "{}", val)?,
//         },
//         Expression::Var(val) => write!(buf, "var_{}", val.index())?,
//         Expression::Operation(cmd, op, op1) => {
//             write!(
//                 buf,
//                 "(var_{} {} var_{})",
//                 op.index(),
//                 cmd.sign(),
//                 op1.index()
//             )?;
//         }
//         Expression::StackOps(ops) => {
//             writeln!(buf, "{{")?;
//             for op in &ops.vec {
//                 print_stack_op(op, buf, width + 2)?;
//                 writeln!(buf, ";")?;
//             }
//             write!(buf, "{:width$}}}", " ")?;
//         }
//         Expression::MLoad { memory, offset } => {
//             write!(
//                 buf,
//                 "var_{:?}.mem_load(var_{:?})",
//                 memory.index(),
//                 offset.index()
//             )?;
//         }
//         Expression::SLoad {
//             storage,
//             key: offset,
//         } => {
//             write!(
//                 buf,
//                 "var_{:?}.state_load(var_{:?})",
//                 storage.index(),
//                 offset.index()
//             )?;
//         }
//         Expression::MSize { memory } => {
//             write!(buf, "var_{:?}.mem_len()", memory.index())?;
//         }
//         Expression::GetMem => {
//             write!(buf, "contract_memory()")?;
//         }
//         Expression::GetStore => {
//             write!(buf, "borrow_storage()")?;
//         }
//         Expression::Cast(var, cast) => {
//             write!(buf, "var_{:?} as {:?}", var.index(), cast.to())?;
//         }
//         Expression::MSlice {
//             memory,
//             offset,
//             len,
//         } => {
//             write!(
//                 buf,
//                 "var_{:?}.mem_slice(var_{:?}, var_{:?})",
//                 memory.index(),
//                 offset.index(),
//                 len.index()
//             )?;
//         }
//         Expression::BytesLen(bytes) => {
//             write!(buf, "var_{:?}.len()", bytes.index())?;
//         }
//         Expression::ReadNum { data, offset } => {
//             write!(
//                 buf,
//                 "var_{:?}.read_num(var_{:?})",
//                 data.index(),
//                 offset.index()
//             )?;
//         }
//         Expression::Hash { mem, offset, len } => {
//             write!(
//                 buf,
//                 "var_{:?}.hash(var_{:?}, var_{:?})",
//                 mem.index(),
//                 offset.index(),
//                 len.index()
//             )?;
//         }
//     }
//     Ok(())
// }
//
// fn print_stack_op(op: &StackOp, buf: &mut String, width: usize) -> Result<(), Error> {
//     match op {
//         StackOp::PushBoolVar(val) => {
//             write!(buf, "{:width$}push var_{}", " ", val.index())?;
//         }
//         StackOp::Not => {
//             write!(buf, "{:width$}!", " ")?;
//         }
//         StackOp::PushBool(val) => {
//             write!(buf, "{:width$}push {}", " ", val)?;
//         }
//         StackOp::Eq => {
//             write!(buf, "{:width$}eq", " ")?;
//         }
//     }
//     Ok(())
// }
//
// impl Operation {
//     pub fn sign(&self) -> &str {
//         match self {
//             Operation::Add => "+",
//             Operation::Sub => "-",
//             Operation::Mul => "*",
//             Operation::Div => "/",
//             Operation::Mod => "%",
//             Operation::Lt => "<",
//             Operation::Gt => ">",
//             Operation::Shr => ">>",
//             Operation::Shl => "<<",
//             Operation::BitXor => "^",
//             Operation::BitOr => "|",
//             Operation::BitAnd => "&",
//             Operation::Eq => "==",
//             Operation::Sar => "sar",
//             Operation::Byte => "byte",
//             Operation::SDiv => "sdiv",
//             Operation::SLt => "slt",
//             Operation::SGt => "sgt",
//             Operation::SMod => "smod",
//             Operation::Exp => "exp",
//             Operation::SignExtend => "signextend",
//             Operation::IsZero => " == 0",
//             Operation::BitNot => "!",
//         }
//     }
// }
