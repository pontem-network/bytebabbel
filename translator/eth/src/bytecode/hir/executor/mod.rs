use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::executor::call::CallOp;
use crate::bytecode::hir::executor::code::CodeOp;
use crate::bytecode::hir::executor::control_flow::ControlFlow;
use crate::bytecode::hir::executor::dependency::{Address, Sha3, TxMeta};
use crate::bytecode::hir::executor::event::EventOp;
use crate::bytecode::hir::executor::math::{BinaryOp, TernaryOp, UnaryOp};
use crate::bytecode::hir::executor::memory::MemoryOp;
use crate::bytecode::hir::executor::stack::StackOp;
use crate::bytecode::hir::executor::storage::StorageOp;
use crate::bytecode::hir::ir::_Expr;
use crate::bytecode::instruction::Instruction;
use crate::bytecode::loc::Loc;
use crate::{BlockId, Hir, OpCode};

pub mod call;
pub mod code;
pub mod control_flow;
pub mod dependency;
pub mod event;
pub mod math;
pub mod memory;
pub mod stack;
pub mod storage;

pub trait InstructionHandler {
    fn handle(
        &self,
        params: Vec<Loc<_Expr>>,
        ir: &mut Hir,
        context: &mut Context,
    ) -> ExecutionResult;
}

struct NoOp;

impl InstructionHandler for NoOp {
    fn handle(&self, _: Vec<Loc<_Expr>>, _: &mut Hir, _: &mut Context) -> ExecutionResult {
        ExecutionResult::None
    }
}

impl InstructionHandler for Instruction {
    fn handle(
        &self,
        params: Vec<Loc<_Expr>>,
        ir: &mut Hir,
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
            OpCode::AddMod => TernaryOp::AddMod.handle(params, ir, context),
            OpCode::MulMod => TernaryOp::MulMod.handle(params, ir, context),
            OpCode::Exp => BinaryOp::Exp.handle(params, ir, context),
            OpCode::SignExtend => BinaryOp::SignExtend.handle(params, ir, context),
            OpCode::Lt => BinaryOp::Lt.handle(params, ir, context),
            OpCode::Gt => BinaryOp::Gt.handle(params, ir, context),
            OpCode::SLt => BinaryOp::SLt.handle(params, ir, context),
            OpCode::SGt => BinaryOp::SGt.handle(params, ir, context),
            OpCode::EQ => BinaryOp::Eq.handle(params, ir, context),
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
            OpCode::Dup(_) | OpCode::Swap(_) => unreachable!(),
            OpCode::Pop => StackOp::Pop.handle(params, ir, context),

            OpCode::Log(size) => EventOp(*size).handle(params, ir, context),

            OpCode::Call => CallOp::Call.handle(params, ir, context),
            OpCode::CallCode => CallOp::CallCode.handle(params, ir, context),
            OpCode::DelegateCall => CallOp::DelegateCall.handle(params, ir, context),
            OpCode::StaticCall => CallOp::StaticCall.handle(params, ir, context),

            OpCode::Return => ControlFlow::Return.handle(params, ir, context),
            OpCode::Jump => ControlFlow::Jump.handle(params, ir, context),
            OpCode::JumpIf => ControlFlow::JumpIf(self.clone()).handle(params, ir, context),
            OpCode::Revert => ControlFlow::Revert.handle(params, ir, context),
            OpCode::Stop => ControlFlow::Stop.handle(params, ir, context),
            OpCode::Invalid(val) => ControlFlow::Abort(*val).handle(params, ir, context),
            OpCode::SelfDestruct => ControlFlow::Abort(1).handle(params, ir, context),
        }
    }
}

pub enum ExecutionResult {
    None,
    End,
    Output(_Expr),
    Jmp(BlockId),
    CndJmp {
        cnd: Loc<_Expr>,
        true_br: BlockId,
        false_br: BlockId,
    },
}
