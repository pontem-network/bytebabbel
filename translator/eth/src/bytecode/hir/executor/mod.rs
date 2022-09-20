pub mod call;
pub mod code;
pub mod control_flow;
pub mod dependency;
pub mod event;
pub mod math;
pub mod memory;
pub mod stack;
pub mod storage;

use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::executor::call::CallOp;
use crate::bytecode::hir::executor::code::CodeOp;
use crate::bytecode::hir::executor::control_flow::ControlFlow;
use crate::bytecode::hir::executor::dependency::{
    Address, CallDataLoad, CallDataSize, Sha3, TxMeta,
};
use crate::bytecode::hir::executor::event::EventOp;
use crate::bytecode::hir::executor::math::{BinaryOp, TernaryOp, UnaryOp};
use crate::bytecode::hir::executor::memory::MemoryOp;
use crate::bytecode::hir::executor::stack::StackOp;
use crate::bytecode::hir::executor::storage::StorageOp;
use crate::bytecode::hir::ir::expression::Expr;
use crate::bytecode::hir::ir::statement::Statement;
use crate::bytecode::instruction::Instruction;
use crate::{BlockId, OpCode};

pub trait InstructionHandler {
    fn handle(&self, params: Vec<Expr>, ctx: &mut Context) -> ExecutionResult;
}

struct NoOp;

impl InstructionHandler for NoOp {
    fn handle(&self, _: Vec<Expr>, _: &mut Context) -> ExecutionResult {
        ExecutionResult::None
    }
}

impl InstructionHandler for Instruction {
    fn handle(&self, params: Vec<Expr>, context: &mut Context) -> ExecutionResult {
        match &self.1 {
            OpCode::Add => BinaryOp::Add.handle(params, context),
            OpCode::Mul => BinaryOp::Mul.handle(params, context),
            OpCode::Sub => BinaryOp::Sub.handle(params, context),
            OpCode::Div => BinaryOp::Div.handle(params, context),
            OpCode::SDiv => BinaryOp::SDiv.handle(params, context),
            OpCode::Mod => BinaryOp::Mod.handle(params, context),
            OpCode::SMod => BinaryOp::SMod.handle(params, context),
            OpCode::AddMod => TernaryOp::AddMod.handle(params, context),
            OpCode::MulMod => TernaryOp::MulMod.handle(params, context),
            OpCode::Exp => BinaryOp::Exp.handle(params, context),
            OpCode::SignExtend => BinaryOp::SignExtend.handle(params, context),
            OpCode::Lt => BinaryOp::Lt.handle(params, context),
            OpCode::Gt => BinaryOp::Gt.handle(params, context),
            OpCode::SLt => BinaryOp::SLt.handle(params, context),
            OpCode::SGt => BinaryOp::SGt.handle(params, context),
            OpCode::EQ => BinaryOp::Eq.handle(params, context),
            OpCode::And => BinaryOp::And.handle(params, context),
            OpCode::Or => BinaryOp::Or.handle(params, context),
            OpCode::Xor => BinaryOp::Xor.handle(params, context),
            OpCode::Byte => BinaryOp::Byte.handle(params, context),
            OpCode::Shl => BinaryOp::Shl.handle(params, context),
            OpCode::Shr => BinaryOp::Shr.handle(params, context),
            OpCode::Sar => BinaryOp::Sar.handle(params, context),

            OpCode::IsZero => UnaryOp::IsZero.handle(params, context),
            OpCode::Not => UnaryOp::Not.handle(params, context),

            OpCode::Sha3 => Sha3.handle(params, context),
            OpCode::Addr => Address.handle(params, context),

            OpCode::JumpDest => NoOp.handle(params, context),

            OpCode::Balance => TxMeta::Balance.handle(params, context),
            OpCode::Origin => TxMeta::Origin.handle(params, context),
            OpCode::Caller => TxMeta::Caller.handle(params, context),
            OpCode::CallValue => TxMeta::CallValue.handle(params, context),
            OpCode::CallDataLoad => CallDataLoad.handle(params, context),
            OpCode::CallDataSize => CallDataSize.handle(params, context),
            OpCode::Blockhash => TxMeta::Blockhash.handle(params, context),
            OpCode::Timestamp => TxMeta::Timestamp.handle(params, context),
            OpCode::GasLimit => TxMeta::GasLimit.handle(params, context),
            OpCode::Difficulty => TxMeta::Difficulty.handle(params, context),
            OpCode::Number => TxMeta::Number.handle(params, context),
            OpCode::GasPrice => TxMeta::GasPrice.handle(params, context),
            OpCode::Coinbase => TxMeta::Coinbase.handle(params, context),
            OpCode::Gas => TxMeta::Gas.handle(params, context),

            OpCode::CodeSize => CodeOp::CodeSize.handle(params, context),
            OpCode::CallDataCopy => CodeOp::CallDataCopy.handle(params, context),
            OpCode::CodeCopy => CodeOp::CodeCopy.handle(params, context),
            OpCode::ExtCodeSize => CodeOp::ExtCodeSize.handle(params, context),
            OpCode::ExtCodeCopy => CodeOp::ExtCodeCopy.handle(params, context),
            OpCode::ReturnDataSize => CodeOp::ReturnDataSize.handle(params, context),
            OpCode::ReturnDataCopy => CodeOp::ReturnDataCopy.handle(params, context),
            OpCode::ExtCodeHash => CodeOp::ExtCodeHash.handle(params, context),
            OpCode::PC => CodeOp::PC.handle(params, context),
            OpCode::Create => CodeOp::Create.handle(params, context),
            OpCode::Create2 => CodeOp::Create2.handle(params, context),

            OpCode::MLoad => MemoryOp::MLoad.handle(params, context),
            OpCode::MStore => MemoryOp::MStore.handle(params, context),
            OpCode::MStore8 => MemoryOp::MStore8.handle(params, context),
            OpCode::MSize => MemoryOp::MSize.handle(params, context),

            OpCode::SLoad => StorageOp::SLoad.handle(params, context),
            OpCode::SStore => StorageOp::SStore.handle(params, context),

            OpCode::Push(val) => StackOp::Push(val.to_vec()).handle(params, context),
            OpCode::Dup(val) => StackOp::Dup(*val).handle(params, context),
            OpCode::Swap(val) => StackOp::Swap(*val).handle(params, context),
            OpCode::Pop => StackOp::Pop.handle(params, context),

            OpCode::Log(size) => EventOp(*size).handle(params, context),

            OpCode::Call => CallOp::Call.handle(params, context),
            OpCode::CallCode => CallOp::CallCode.handle(params, context),
            OpCode::DelegateCall => CallOp::DelegateCall.handle(params, context),
            OpCode::StaticCall => CallOp::StaticCall.handle(params, context),

            OpCode::Return => ControlFlow::Return.handle(params, context),
            OpCode::Jump => ControlFlow::Jump.handle(params, context),
            OpCode::JumpIf => ControlFlow::JumpIf(self.clone()).handle(params, context),
            OpCode::Revert => ControlFlow::Revert.handle(params, context),
            OpCode::Stop => ControlFlow::Stop.handle(params, context),
            OpCode::Invalid(val) => ControlFlow::Abort(*val).handle(params, context),
            OpCode::SelfDestruct => ControlFlow::Abort(1).handle(params, context),
        }
    }
}

pub enum ExecutionResult {
    Abort(u8),
    Statement(Statement),
    None,
    Expr(Vec<Expr>),
    Result {
        offset: Expr,
        len: Expr,
    },
    Stop,
    Jmp(Expr, BlockId),
    CndJmp {
        cnd: Expr,
        true_br: BlockId,
        false_br: BlockId,
    },
}

impl<T: Into<Expr>> From<T> for ExecutionResult {
    fn from(expr: T) -> Self {
        ExecutionResult::Expr(vec![expr.into()])
    }
}

impl From<Statement> for ExecutionResult {
    fn from(stmt: Statement) -> Self {
        ExecutionResult::Statement(stmt)
    }
}
