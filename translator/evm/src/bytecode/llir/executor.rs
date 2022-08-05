use crate::bytecode::instruction::Instruction;
use crate::bytecode::llir::context::Context;
use crate::bytecode::llir::ops::{BinaryOp, UnaryOp};
use crate::bytecode::llir::stack::{Frame, StackFrame, FRAME_SIZE};
use crate::bytecode::types::U256;
use crate::{BlockId, OpCode};
use anyhow::Error;

pub fn execute(inst: &Instruction, params: Vec<StackFrame>, ctx: &mut Context) -> ExecutionResult {
    match &inst.1 {
        OpCode::Push(val) => ExecutionResult::Output(vec![StackFrame::new(Frame::Val(
            U256::from(val.as_slice()),
        ))]),
        OpCode::Addr => ExecutionResult::Output(vec![StackFrame::new(Frame::SelfAddress)]),
        OpCode::EQ => eq(params, ctx),
        OpCode::MStore => mem_store(params, ctx, true),
        OpCode::MStore8 => mem_store(params, ctx, false),
        OpCode::CallDataSize => ExecutionResult::Output(vec![StackFrame::new(Frame::Val(
            ctx.env().call_data_size(),
        ))]),
        OpCode::Lt => math(params, BinaryOp::Lt, |a, b| Frame::Bool(a < b)),
        OpCode::Gt => math(params, BinaryOp::Gt, |a, b| Frame::Bool(a > b)),
        OpCode::JumpIf => jmp_if(inst, params),
        OpCode::Jump => jmp(params),
        OpCode::CallDataLoad => call_data_load(params, ctx),
        OpCode::Shr => math(params, BinaryOp::Shr, |a, b| Frame::Val(b >> a)),
        OpCode::Stop => stop(),
        OpCode::Add => math(params, BinaryOp::Add, |a, b| {
            Frame::Val(b.overflowing_add(a).0)
        }),
        OpCode::Mul => math(params, BinaryOp::Mul, |a, b| {
            Frame::Val(b.overflowing_mul(a).0)
        }),
        OpCode::Sub => math(params, BinaryOp::Sub, |a, b| {
            Frame::Val(a.overflowing_sub(b).0)
        }),
        OpCode::Div => math(params, BinaryOp::Div, |a, b| {
            Frame::Val(a.checked_div(b).unwrap_or_else(U256::zero))
        }),
        OpCode::SDiv => todo!(),
        OpCode::Mod => todo!(),
        OpCode::SMod => todo!(),
        OpCode::AddMod => todo!(),
        OpCode::MulMod => todo!(),
        OpCode::Exp => todo!(),
        OpCode::SignExtend => todo!(),
        OpCode::SLt => math(params, BinaryOp::SLt, |a, b| {
            Frame::Bool((a.as_usize() as isize) < (b.as_usize() as isize))
        }),
        OpCode::SGt => todo!(),
        OpCode::IsZero => is_zero(params, ctx),
        OpCode::And => math(params, BinaryOp::And, |a, b| Frame::Val(a & b)),
        OpCode::Or => todo!(),
        OpCode::Xor => todo!(),
        OpCode::Not => todo!(),
        OpCode::Byte => math(params, BinaryOp::Byte, |i, x| {
            Frame::Val((x >> (U256::from(248) - i * U256::from(8))) & U256::from(0xFF))
        }),
        OpCode::Shl => todo!(),
        OpCode::Sar => todo!(),
        OpCode::Sha3 => todo!(),
        OpCode::Balance => todo!(),
        OpCode::Origin => todo!(),
        OpCode::Caller => todo!(),
        OpCode::CallValue => call_value(),
        OpCode::CallDataCopy => todo!(),
        OpCode::CodeSize => todo!(),
        OpCode::CodeCopy => code_copy(params),
        OpCode::GasPrice => todo!(),
        OpCode::ExtCodeSize => todo!(),
        OpCode::ExtCodeCopy => todo!(),
        OpCode::ReturnDataSize => todo!(),
        OpCode::ReturnDataCopy => todo!(),
        OpCode::ExtCodeHash => todo!(),
        OpCode::Blockhash => todo!(),
        OpCode::Coinbase => todo!(),
        OpCode::Timestamp => todo!(),
        OpCode::Number => todo!(),
        OpCode::Difficulty => todo!(),
        OpCode::GasLimit => todo!(),
        OpCode::Pop => ExecutionResult::None,
        OpCode::MLoad => mem_load(params, ctx),
        OpCode::SLoad => todo!(),
        OpCode::SStore => todo!(),
        OpCode::PC => todo!(),
        OpCode::MSize => todo!(),
        OpCode::Gas => todo!(),
        OpCode::JumpDest => ExecutionResult::None,
        OpCode::Dup(_) => dup(params),
        OpCode::Swap(_) => swap(params),
        OpCode::Log(_) => todo!(),
        OpCode::Create => todo!(),
        OpCode::Call => todo!(),
        OpCode::CallCode => todo!(),
        OpCode::Return => res(params),
        OpCode::DelegateCall => todo!(),
        OpCode::Create2 => todo!(),
        OpCode::Revert => revert(params),
        OpCode::StaticCall => todo!(),
        OpCode::Invalid(code) => abort(*code),
        OpCode::SelfDestruct => abort(1),
    }
}

fn is_zero(params: Vec<StackFrame>, ctx: &mut Context) -> ExecutionResult {
    let a = &params[0];
    let a = a.as_u256();
    let mut frame = if let Some(a) = a {
        StackFrame::new(Frame::Bool(a.is_zero()))
    } else {
        StackFrame::new(Frame::Calc(UnaryOp::IsZero, params[0].clone()))
    };

    ExecutionResult::Output(vec![frame])
}

fn eq(params: Vec<StackFrame>, ctx: &mut Context) -> ExecutionResult {
    let mut a = params[0].clone();
    let mut b = params[1].clone();
    if a == b {
        let mut new = StackFrame::new(Frame::Bool(true));
        ExecutionResult::Output(vec![new])
    } else {
        math(params, BinaryOp::EQ, |a, b| Frame::Bool(a == b))
    }
}

fn math<F: Fn(U256, U256) -> Frame>(
    params: Vec<StackFrame>,
    op: BinaryOp,
    f: F,
) -> ExecutionResult {
    let a = &params[0];
    let a = a.as_u256();

    let b = &params[1];
    let b = b.as_u256();

    if let Some(a) = a {
        if let Some(b) = b {
            let mut new = StackFrame::new(f(a, b));
            return ExecutionResult::Output(vec![new]);
        }
    }

    let mut new = StackFrame::new(Frame::Calc2(op, params[0].clone(), params[1].clone()));
    ExecutionResult::Output(vec![new])
}

fn mem_store(params: Vec<StackFrame>, ctx: &mut Context, full: bool) -> ExecutionResult {
    let mut rf = params[0].clone();

    let mut val = params[1].clone();
    let val = if full { val } else { todo!() };
    todo!()
    //ctx.mem_store(rf, val);
    // ExecutionResult::None
}

fn mem_load(params: Vec<StackFrame>, ctx: &mut Context) -> ExecutionResult {
    let addr = params[0].clone();
    //let res = ctx.mem_load(&addr);
    todo!()
    //ExecutionResult::Output(vec![res])
}

fn jmp_if(inst: &Instruction, mut params: Vec<StackFrame>) -> ExecutionResult {
    let destination = &params[0];
    let true_br = destination.as_block_id().expect("TODO cast to block id.");
    let false_br = BlockId(inst.0 + inst.1.size());
    let cnd = params.remove(1);
    if let Some(cnd) = cnd.as_bool() {
        if cnd {
            ExecutionResult::Jmp(true_br)
        } else {
            ExecutionResult::Jmp(BlockId(inst.0 + inst.1.size()))
        }
    } else {
        ExecutionResult::CndJmp {
            cnd,
            true_br,
            false_br,
        }
    }
}

fn jmp(params: Vec<StackFrame>) -> ExecutionResult {
    let destination = &params[0];
    let dest = destination.as_block_id().expect("TODO cast to block id.");
    ExecutionResult::Jmp(dest)
}

fn call_data_load(params: Vec<StackFrame>, ctx: &mut Context) -> ExecutionResult {
    if let Some(offset) = params[0].as_u256() {
        if offset.is_zero() {
            let mut buf = [0u8; 32];
            buf[0..4].copy_from_slice(ctx.env().hash().as_ref().as_slice());
            let new_frame = StackFrame::new(Frame::Val(U256::from(buf)));
            ExecutionResult::Output(vec![new_frame])
        } else {
            let index = (offset.as_usize() - 4) / FRAME_SIZE;
            let new_frame = StackFrame::new(Frame::Param(index as u16));
            ExecutionResult::Output(vec![new_frame])
        }
    } else {
        panic!("Unsupported dynamic call data load");
    }
}

fn dup(params: Vec<StackFrame>) -> ExecutionResult {
    let mut out = params;
    let new_item = out[out.len() - 1].clone();
    out.insert(0, new_item);
    ExecutionResult::Output(out)
}

fn swap(params: Vec<StackFrame>) -> ExecutionResult {
    let mut out = params;
    let last_index = out.len() - 1;
    out.swap(0, last_index);
    ExecutionResult::Output(out)
}

fn abort(code: u8) -> ExecutionResult {
    ExecutionResult::Abort(code)
}

fn stop() -> ExecutionResult {
    ExecutionResult::Stop
}

fn revert(mut params: Vec<StackFrame>) -> ExecutionResult {
    let len = params.remove(1);
    let offset = params.remove(0);
    ExecutionResult::Result {
        offset,
        len,
        revert: true,
    }
}

fn res(mut params: Vec<StackFrame>) -> ExecutionResult {
    let len = params.remove(1);
    let offset = params.remove(0);
    ExecutionResult::Result {
        offset,
        len,
        revert: false,
    }
}

fn code_copy(mut params: Vec<StackFrame>) -> ExecutionResult {
    let offset = params
        .remove(1)
        .as_block_id()
        .expect("Unsupported code copy dynamic offset.");
    ExecutionResult::CodeCopy(offset)
}

fn call_value() -> ExecutionResult {
    // todo call value.
    ExecutionResult::Output(vec![StackFrame::new(Frame::Val(U256::from(0)))])
}

pub enum ExecutionResult {
    CodeCopy(BlockId),
    Abort(u8),
    None,
    Output(Vec<StackFrame>),
    Result {
        offset: StackFrame,
        len: StackFrame,
        revert: bool,
    },
    Stop,
    Jmp(BlockId),
    CndJmp {
        cnd: StackFrame,
        true_br: BlockId,
        false_br: BlockId,
    },
}
