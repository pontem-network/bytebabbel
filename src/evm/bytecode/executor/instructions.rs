use crate::evm::bytecode::executor::stack::{Frame, StackFrame, Used, FRAME_SIZE};
use crate::evm::bytecode::executor::types::U256;
use crate::evm::bytecode::executor::{Context, Jump};
use crate::evm::bytecode::instruction::Instruction;
use crate::evm::OpCode;
use anyhow::Error;

pub fn execute(inst: &Instruction, ctx: &mut Context) -> Result<Vec<StackFrame>, Error> {
    Ok(match &inst.1 {
        OpCode::Push(val) => {
            vec![StackFrame::new(
                ctx.next_id(),
                Frame::Val(U256::from(val.as_slice())),
            )]
        }
        OpCode::Addr => {
            vec![StackFrame::new(ctx.next_id(), Frame::SelfAddress)]
        }
        OpCode::EQ => math(OpCode::EQ, ctx, |a, b| Frame::Bool(a == b)),
        OpCode::MStore => mem_store(ctx, true),
        OpCode::MStore8 => mem_store(ctx, false),
        OpCode::CallDataSize => vec![StackFrame::new(
            ctx.next_id(),
            Frame::Val(ctx.env().call_data_size()),
        )],
        OpCode::Lt => math(OpCode::Lt, ctx, |a, b| Frame::Bool(a < b)),
        OpCode::Gt => math(OpCode::Gt, ctx, |a, b| Frame::Bool(a > b)),
        OpCode::JumpIf => jmp_if(ctx),
        OpCode::Jump => jmp(ctx),
        OpCode::CallDataLoad => call_data_load(ctx),
        OpCode::Shr => math(OpCode::Shr, ctx, |a, b| Frame::Val(b >> a)),
        OpCode::Stop => abort(ctx, 1),
        OpCode::Add => math(OpCode::Add, ctx, |a, b| Frame::Val(b + a)),
        OpCode::Mul => math(OpCode::Mul, ctx, |a, b| Frame::Val(b * a)),
        OpCode::Sub => math(OpCode::Sub, ctx, |a, b| Frame::Val(a - b)),
        OpCode::Div => math(OpCode::Div, ctx, |a, b| Frame::Val(a / b)),
        OpCode::SDiv => todo!(),
        OpCode::Mod => todo!(),
        OpCode::SMod => todo!(),
        OpCode::AddMod => todo!(),
        OpCode::MulMod => todo!(),
        OpCode::Exp => todo!(),
        OpCode::SignExtend => todo!(),
        OpCode::SLt => todo!(),
        OpCode::SGt => todo!(),
        OpCode::IsZero => is_zero(ctx),
        OpCode::And => todo!(),
        OpCode::Or => todo!(),
        OpCode::Xor => todo!(),
        OpCode::Not => todo!(),
        OpCode::Byte => math(OpCode::Byte, ctx, |i, x| {
            Frame::Val((x >> (U256::from(248) - i * U256::from(8))) & U256::from(0xFF))
        }),
        OpCode::Shl => todo!(),
        OpCode::Sar => todo!(),
        OpCode::Sha3 => todo!(),
        OpCode::Balance => todo!(),
        OpCode::Origin => todo!(),
        OpCode::Caller => todo!(),
        OpCode::CallValue => todo!(),
        OpCode::CallDataCopy => todo!(),
        OpCode::CodeSize => todo!(),
        OpCode::CodeCopy => code_copy(ctx),
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
        OpCode::Pop => vec![],
        OpCode::MLoad => mem_load(ctx),
        OpCode::SLoad => todo!(),
        OpCode::SStore => todo!(),
        OpCode::PC => todo!(),
        OpCode::MSize => todo!(),
        OpCode::Gas => todo!(),
        OpCode::JumpDest => vec![],
        OpCode::Dup(_) => dup(ctx),
        OpCode::Swap(_) => swap(ctx),
        OpCode::Log(_) => todo!(),
        OpCode::Create => todo!(),
        OpCode::Call => todo!(),
        OpCode::CallCode => todo!(),
        OpCode::Return => res(ctx),
        OpCode::DelegateCall => todo!(),
        OpCode::Create2 => todo!(),
        OpCode::Revert => abort(ctx, 2),
        OpCode::StaticCall => todo!(),
        OpCode::Invalid(code) => abort(ctx, *code),
        OpCode::SelfDestruct => abort(ctx, 1),
    })
}

fn is_zero(ctx: &mut Context) -> Vec<StackFrame> {
    let used = Used::default();
    let a = &mut ctx.input[0];
    a.set_used_flag(used.clone());
    let a = a.as_u256();
    let mut frame = if let Some(a) = a {
        StackFrame::new(ctx.next_id(), Frame::Bool(a.is_zero()))
    } else {
        StackFrame::new(
            ctx.next_id(),
            Frame::Calc(OpCode::IsZero, ctx.input[0].clone()),
        )
    };

    frame.set_used_flag(used);
    vec![frame]
}

fn math<F: Fn(U256, U256) -> Frame>(op: OpCode, ctx: &mut Context, f: F) -> Vec<StackFrame> {
    let used = Used::default();
    let a = &mut ctx.input[0];
    a.set_used_flag(used.clone());
    let a = a.as_u256();

    let b = &mut ctx.input[1];
    b.set_used_flag(used.clone());
    let b = b.as_u256();

    if let Some(a) = a {
        if let Some(b) = b {
            let mut new = StackFrame::new(ctx.next_id(), f(a, b));
            new.set_used_flag(used);
            return vec![new];
        }
    }

    let mut new = StackFrame::new(
        ctx.next_id(),
        Frame::Calc2(op, ctx.input[0].clone(), ctx.input[1].clone()),
    );
    new.set_used_flag(used);
    vec![new]
}

fn mem_store(ctx: &mut Context, full: bool) -> Vec<StackFrame> {
    let used = Used::default();
    let mut rf = ctx.input[0].clone();
    rf.set_used_flag(used.clone());

    let mut val = ctx.input[1].clone();
    val.set_used_flag(used);
    let val = if full { val } else { todo!() };

    ctx.mem_store(rf, val);
    vec![]
}

fn mem_load(ctx: &mut Context) -> Vec<StackFrame> {
    let addr = ctx.input[0].clone();
    vec![ctx.mem_load(&addr)]
}

fn jmp_if(ctx: &mut Context) -> Vec<StackFrame> {
    let destination = &ctx.input[0];
    destination.mark_as_used();
    let dest = destination.as_block_id().expect("TODO cast to block id.");
    let cnd = &ctx.input[1];
    cnd.mark_as_used();
    if let Some(cnd) = cnd.as_bool() {
        ctx.set_jump(if cnd {
            Jump::UnCnd(dest)
        } else {
            Jump::UnCnd(ctx.next_block)
        });
    } else {
        ctx.set_jump(Jump::Cnd {
            true_br: dest,
            false_br: ctx.next_block,
        })
    }
    vec![]
}

fn jmp(ctx: &mut Context) -> Vec<StackFrame> {
    let destination = &ctx.input[0];
    destination.mark_as_used();
    let dest = destination.as_block_id().expect("TODO cast to block id.");
    ctx.set_jump(Jump::UnCnd(dest));
    vec![]
}

fn call_data_load(ctx: &mut Context) -> Vec<StackFrame> {
    if let Some(offset) = ctx.input[0].as_u256() {
        let index = offset.as_usize() / FRAME_SIZE;
        if index == 0 {
            let mut buf = [0u8; 32];
            buf[0..4].copy_from_slice(ctx.env.hash().as_ref().as_slice());
            let new_frame = StackFrame::new(ctx.next_id(), Frame::Val(U256::from(buf)));
            ctx.input[0].set_used_flag(new_frame.get_used_flag());
            vec![new_frame]
        } else {
            let new_frame = StackFrame::new(ctx.next_id(), Frame::Param(index as u16));
            ctx.input[0].set_used_flag(new_frame.get_used_flag());
            vec![new_frame]
        }
    } else {
        todo!()
    }
}

fn dup(ctx: &mut Context) -> Vec<StackFrame> {
    let mut out = ctx.input.to_vec();
    let new_item = out[out.len() - 1].clone();
    out.insert(0, new_item);
    out
}

fn swap(ctx: &mut Context) -> Vec<StackFrame> {
    let mut out = ctx.input.to_vec();
    let last_index = out.len() - 1;
    out.swap(0, last_index);
    out
}

fn abort(ctx: &mut Context, code: u8) -> Vec<StackFrame> {
    ctx.set_abort(code);
    vec![]
}

fn res(ctx: &mut Context) -> Vec<StackFrame> {
    let offset = &ctx.input[0];
    offset.mark_as_used();
    let len = &ctx.input[1];
    len.mark_as_used();
    ctx.set_result(offset.clone(), len.clone());
    vec![]
}

fn code_copy(ctx: &mut Context) -> Vec<StackFrame> {
    let offset = &ctx.input[1]
        .as_block_id()
        .expect("Unsupported code copy dynamic offset.");
    ctx.set_code_offset(*offset);
    vec![]
}
