pub mod call;
pub mod code;
pub mod control_flow;
pub mod dependency;
pub mod event;
pub mod math;
pub mod memory;
pub mod stack;
pub mod storage;

use crate::bytecode::instruction::Instruction;
use crate::bytecode::llir::context::Context;
use crate::bytecode::llir::executor::call::CallOp;
use crate::bytecode::llir::executor::code::CodeOp;
use crate::bytecode::llir::executor::control_flow::ControlFlow;
use crate::bytecode::llir::executor::dependency::{Address, Sha3, TxMeta};
use crate::bytecode::llir::executor::event::EventOp;
use crate::bytecode::llir::executor::math::{BinaryOp, UnaryOp};
use crate::bytecode::llir::executor::memory::MemoryOp;
use crate::bytecode::llir::executor::stack::StackOp;
use crate::bytecode::llir::executor::storage::StorageOp;
use crate::bytecode::llir::ir::var::Var;
use crate::bytecode::llir::stack::{Frame, StackFrame, FRAME_SIZE};
use crate::bytecode::types::U256;
use crate::{BlockId, Ir, OpCode};

pub trait InstructionHandler {
    fn handle(
        &self,
        params: Vec<StackFrame>,
        ir: &mut Ir,
        context: &mut Context,
    ) -> ExecutionResult;
}

struct NoOp;

impl InstructionHandler for NoOp {
    fn handle(&self, _: Vec<StackFrame>, _: &mut Ir, _: &mut Context) -> ExecutionResult {
        ExecutionResult::None
    }
}

impl InstructionHandler for Instruction {
    fn handle(
        &self,
        params: Vec<StackFrame>,
        ir: &mut Ir,
        context: &mut Context,
    ) -> ExecutionResult {
        match &self.1 {
            OpCode::Add => BinaryOp::Add.handle(params, ir, context),
            OpCode::Mul => BinaryOp::Mul.handle(params, ir, context),
            OpCode::Sub => BinaryOp::Sub.handle(params, ir, context),
            OpCode::Div => BinaryOp::Div.handle(params, ir, context),
            OpCode::SDiv => BinaryOp::SDiv.handle(params, ir, context),
            OpCode::Mod => BinaryOp::Mod.handle(params, ir, context),
            OpCode::SMod => BinaryOp::SMod.handle(params, ir, context),
            OpCode::AddMod => BinaryOp::AddMod.handle(params, ir, context),
            OpCode::MulMod => BinaryOp::MulMod.handle(params, ir, context),
            OpCode::Exp => BinaryOp::Exp.handle(params, ir, context),
            OpCode::SignExtend => BinaryOp::SignExtend.handle(params, ir, context),
            OpCode::Lt => BinaryOp::Lt.handle(params, ir, context),
            OpCode::Gt => BinaryOp::Gt.handle(params, ir, context),
            OpCode::SLt => BinaryOp::SLt.handle(params, ir, context),
            OpCode::SGt => BinaryOp::SGt.handle(params, ir, context),
            OpCode::EQ => BinaryOp::EQ.handle(params, ir, context),
            OpCode::And => BinaryOp::And.handle(params, ir, context),
            OpCode::Or => BinaryOp::Or.handle(params, ir, context),
            OpCode::Xor => BinaryOp::Xor.handle(params, ir, context),
            OpCode::Byte => BinaryOp::Byte.handle(params, ir, context),
            OpCode::Shl => BinaryOp::Shl.handle(params, ir, context),
            OpCode::Shr => BinaryOp::Shr.handle(params, ir, context),
            OpCode::Sar => BinaryOp::Sar.handle(params, ir, context),

            OpCode::IsZero => UnaryOp::IsZero.handle(params, ir, context),
            OpCode::Not => UnaryOp::Not.handle(params, ir, context),

            OpCode::Sha3 => Sha3.handle(params, ir, context),
            OpCode::Addr => Address.handle(params, ir, context),

            OpCode::JumpDest => NoOp.handle(params, ir, context),

            OpCode::Balance => TxMeta::Balance.handle(params, ir, context),
            OpCode::Origin => TxMeta::Balance.handle(params, ir, context),
            OpCode::Caller => TxMeta::Caller.handle(params, ir, context),
            OpCode::CallValue => TxMeta::CallValue.handle(params, ir, context),
            OpCode::CallDataLoad => TxMeta::CallDataLoad.handle(params, ir, context),
            OpCode::CallDataSize => TxMeta::CallDataSize.handle(params, ir, context),
            OpCode::Blockhash => TxMeta::Blockhash.handle(params, ir, context),
            OpCode::Timestamp => TxMeta::Timestamp.handle(params, ir, context),
            OpCode::GasLimit => TxMeta::GasLimit.handle(params, ir, context),
            OpCode::Difficulty => TxMeta::Difficulty.handle(params, ir, context),
            OpCode::Number => TxMeta::Number.handle(params, ir, context),
            OpCode::GasPrice => TxMeta::GasPrice.handle(params, ir, context),
            OpCode::Coinbase => TxMeta::Coinbase.handle(params, ir, context),
            OpCode::Gas => TxMeta::Gas.handle(params, ir, context),

            OpCode::CodeSize => CodeOp::CodeSize.handle(params, ir, context),
            OpCode::CallDataCopy => CodeOp::CallDataCopy.handle(params, ir, context),
            OpCode::CodeCopy => CodeOp::CodeCopy.handle(params, ir, context),
            OpCode::ExtCodeSize => CodeOp::ExtCodeSize.handle(params, ir, context),
            OpCode::ExtCodeCopy => CodeOp::ExtCodeCopy.handle(params, ir, context),
            OpCode::ReturnDataSize => CodeOp::ReturnDataSize.handle(params, ir, context),
            OpCode::ReturnDataCopy => CodeOp::ReturnDataCopy.handle(params, ir, context),
            OpCode::ExtCodeHash => CodeOp::ExtCodeHash.handle(params, ir, context),
            OpCode::PC => CodeOp::PC.handle(params, ir, context),
            OpCode::Create => CodeOp::Create.handle(params, ir, context),
            OpCode::Create2 => CodeOp::Create2.handle(params, ir, context),

            OpCode::MLoad => MemoryOp::MLoad.handle(params, ir, context),
            OpCode::MStore => MemoryOp::MStore.handle(params, ir, context),
            OpCode::MStore8 => MemoryOp::MStore8.handle(params, ir, context),
            OpCode::MSize => MemoryOp::MSize.handle(params, ir, context),

            OpCode::SLoad => StorageOp::SLoad.handle(params, ir, context),
            OpCode::SStore => StorageOp::SStore.handle(params, ir, context),

            OpCode::Push(val) => StackOp::Push(val.to_vec()).handle(params, ir, context),
            OpCode::Dup(val) => StackOp::Dup(*val).handle(params, ir, context),
            OpCode::Swap(val) => StackOp::Swap(*val).handle(params, ir, context),
            OpCode::Pop => StackOp::Pop.handle(params, ir, context),

            OpCode::Log(size) => EventOp(*size).handle(params, ir, context),

            OpCode::Call => CallOp::Call.handle(params, ir, context),
            OpCode::CallCode => CallOp::CallCode.handle(params, ir, context),
            OpCode::DelegateCall => CallOp::DelegateCall.handle(params, ir, context),
            OpCode::StaticCall => CallOp::StaticCall.handle(params, ir, context),

            OpCode::Return => ControlFlow::Return.handle(params, ir, context),
            OpCode::Jump => ControlFlow::Jump.handle(params, ir, context),
            OpCode::JumpIf => ControlFlow::JumpIf.handle(params, ir, context),
            OpCode::Revert => ControlFlow::Revert.handle(params, ir, context),
            OpCode::Stop => ControlFlow::Stop.handle(params, ir, context),
            OpCode::Invalid(val) => ControlFlow::Abort(*val).handle(params, ir, context),
            OpCode::SelfDestruct => ControlFlow::Abort(1).handle(params, ir, context),
        }
    }
}

pub fn execute(
    inst: &Instruction,
    params: Vec<StackFrame>,
    ir: &mut Ir,
    ctx: &mut Context,
) -> ExecutionResult {
    inst.handle(params, ir, ctx)
    // match &inst.1 {
    //     OpCode::Push(val) => {
    //         let id = ir.create_var(Var::Val(U256::from(val.as_slice())));
    //         ExecutionResult::Output(vec![StackFrame::new(Frame::Var(id))])
    //     }
    //     OpCode::Addr => ExecutionResult::Output(vec![StackFrame::new(Frame::SelfAddress)]),
    //     OpCode::EQ => eq(params, ctx),
    //     OpCode::MStore => mem_store(params, ctx, true),
    //     OpCode::MStore8 => mem_store(params, ctx, false),
    //     OpCode::CallDataSize => {
    //         let id = ir.create_var(Var::Val(ctx.env().call_data_size()));
    //         ExecutionResult::Output(vec![StackFrame::new(Frame::Var(id))])
    //     }
    //     OpCode::Lt => math(params, BinaryOp::Lt, |a, b| Frame::Bool(a < b)),
    //     OpCode::Gt => math(params, BinaryOp::Gt, |a, b| Frame::Bool(a > b)),
    //     OpCode::JumpIf => jmp_if(inst, params),
    //     OpCode::Jump => jmp(params),
    //     OpCode::CallDataLoad => call_data_load(params, ctx),
    //     OpCode::Shr => math(params, BinaryOp::Shr, |a, b| Frame::Var(b >> a)),
    //     OpCode::Stop => stop(),
    //     OpCode::Add => math(params, BinaryOp::Add, |a, b| {
    //         Frame::Var(b.overflowing_add(a).0)
    //     }),
    //     OpCode::Mul => math(params, BinaryOp::Mul, |a, b| {
    //         Frame::Var(b.overflowing_mul(a).0)
    //     }),
    //     OpCode::Sub => math(params, BinaryOp::Sub, |a, b| {
    //         Frame::Var(a.overflowing_sub(b).0)
    //     }),
    //     OpCode::Div => math(params, BinaryOp::Div, |a, b| {
    //         Frame::Var(a.checked_div(b).unwrap_or_else(U256::zero))
    //     }),
    //     OpCode::SDiv => todo!(),
    //     OpCode::Mod => todo!(),
    //     OpCode::SMod => todo!(),
    //     OpCode::AddMod => todo!(),
    //     OpCode::MulMod => todo!(),
    //     OpCode::Exp => todo!(),
    //     OpCode::SignExtend => todo!(),
    //     OpCode::SLt => math(params, BinaryOp::SLt, |a, b| {
    //         Frame::Bool((a.as_usize() as isize) < (b.as_usize() as isize))
    //     }),
    //     OpCode::SGt => todo!(),
    //     OpCode::IsZero => is_zero(params, ctx),
    //     OpCode::And => math(params, BinaryOp::And, |a, b| Frame::Var(a & b)),
    //     OpCode::Or => todo!(),
    //     OpCode::Xor => todo!(),
    //     OpCode::Not => todo!(),
    //     OpCode::Byte => math(params, BinaryOp::Byte, |i, x| {
    //         Frame::Var((x >> (U256::from(248) - i * U256::from(8))) & U256::from(0xFF))
    //     }),
    //     OpCode::Shl => todo!(),
    //     OpCode::Sar => todo!(),
    //     OpCode::Sha3 => todo!(),
    //     OpCode::Balance => todo!(),
    //     OpCode::Origin => todo!(),
    //     OpCode::Caller => todo!(),
    //     OpCode::CallValue => call_value(ir),
    //     OpCode::CallDataCopy => todo!(),
    //     OpCode::CodeSize => todo!(),
    //     OpCode::CodeCopy => code_copy(params),
    //     OpCode::GasPrice => todo!(),
    //     OpCode::ExtCodeSize => todo!(),
    //     OpCode::ExtCodeCopy => todo!(),
    //     OpCode::ReturnDataSize => todo!(),
    //     OpCode::ReturnDataCopy => todo!(),
    //     OpCode::ExtCodeHash => todo!(),
    //     OpCode::Blockhash => todo!(),
    //     OpCode::Coinbase => todo!(),
    //     OpCode::Timestamp => todo!(),
    //     OpCode::Number => todo!(),
    //     OpCode::Difficulty => todo!(),
    //     OpCode::GasLimit => todo!(),
    //     OpCode::Pop => ExecutionResult::None,
    //     OpCode::MLoad => mem_load(params, ctx),
    //     OpCode::SLoad => todo!(),
    //     OpCode::SStore => todo!(),
    //     OpCode::PC => todo!(),
    //     OpCode::MSize => todo!(),
    //     OpCode::Gas => todo!(),
    //     OpCode::JumpDest => ExecutionResult::None,
    //     OpCode::Dup(_) => dup(params),
    //     OpCode::Swap(_) => swap(params),
    //     OpCode::Log(_) => todo!(),
    //     OpCode::Create => todo!(),
    //     OpCode::Call => todo!(),
    //     OpCode::CallCode => todo!(),
    //     OpCode::Return => res(params),
    //     OpCode::DelegateCall => todo!(),
    //     OpCode::Create2 => todo!(),
    //     OpCode::Revert => revert(params),
    //     OpCode::StaticCall => todo!(),
    //     OpCode::Invalid(code) => abort(*code),
    //     OpCode::SelfDestruct => abort(1),
    // }
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
    todo!()
    // if let Some(offset) = params[0].as_u256() {
    //     if offset.is_zero() {
    //         let mut buf = [0u8; 32];
    //         buf[0..4].copy_from_slice(ctx.env().hash().as_ref().as_slice());
    //         let new_frame = StackFrame::new(Frame::Var(U256::from(buf)));
    //         ExecutionResult::Output(vec![new_frame])
    //     } else {
    //         let index = (offset.as_usize() - 4) / FRAME_SIZE;
    //         let new_frame = StackFrame::new(Frame::Param(index as u16));
    //         ExecutionResult::Output(vec![new_frame])
    //     }
    // } else {
    //     panic!("Unsupported dynamic call data load");
    // }
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

fn code_copy(mut params: Vec<StackFrame>) -> ExecutionResult {
    let offset = params
        .remove(1)
        .as_block_id()
        .expect("Unsupported code copy dynamic offset.");
    ExecutionResult::CodeCopy(offset)
}

fn call_value(ir: &mut Ir) -> ExecutionResult {
    // todo call value.
    let id = ir.create_var(Var::Val(U256::from(0)));
    ExecutionResult::Output(vec![StackFrame::new(Frame::Var(id))])
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
