use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir::ir::var::{Eval, VarId};
use crate::Hir;
use primitive_types::U256;

pub struct Sha3;

impl InstructionHandler for Sha3 {
    fn handle(&self, params: Vec<VarId>, ir: &mut Hir, _: &mut Context) -> ExecutionResult {
        let id = ir.create_var(Eval::Hash(params[0], params[1]));
        ExecutionResult::Output(vec![id])
    }
}

pub struct Address;

impl InstructionHandler for Address {
    fn handle(&self, _: Vec<VarId>, ir: &mut Hir, ctx: &mut Context) -> ExecutionResult {
        let id = ir.create_var(Eval::Val(ctx.address()));
        ExecutionResult::Output(vec![id])
    }
}

pub enum TxMeta {
    Balance,
    Origin,
    Caller,
    CallValue,
    CallDataLoad,
    CallDataSize,
    Blockhash,
    Timestamp,
    GasLimit,
    Difficulty,
    Number,
    GasPrice,
    Coinbase,
    Gas,
}

impl InstructionHandler for TxMeta {
    fn handle(&self, params: Vec<VarId>, ir: &mut Hir, ctx: &mut Context) -> ExecutionResult {
        let val = match self {
            TxMeta::Balance => U256::zero(),
            TxMeta::Origin => U256::zero(),
            TxMeta::Caller => {
                let id = ir.create_var(Eval::Signer);
                return ExecutionResult::Output(vec![id]);
            }
            TxMeta::CallValue => U256::zero(),
            TxMeta::CallDataLoad => {
                return ExecutionResult::Output(vec![id]);
            }
            TxMeta::CallDataSize => {
                return ExecutionResult::Output(vec![call_data_size(ir, ctx)]);
            }
            TxMeta::Blockhash => U256::zero(),
            TxMeta::Timestamp => U256::zero(),
            TxMeta::Difficulty => U256::zero(),
            TxMeta::Number => U256::zero(),
            TxMeta::GasPrice => U256::zero(),
            TxMeta::Coinbase => U256::zero(),
            TxMeta::GasLimit => U256::MAX,
            TxMeta::Gas => U256::MAX,
        };
        let id = ir.create_var(Eval::Val(val));
        ExecutionResult::Output(vec![id])
    }
}

fn call_data_size(ir: &mut Hir, ctx: &mut Context) -> VarId {
    if ctx.flags().native_input {
        ir.create_var(Eval::Val(ctx.env().call_data_size()))
    } else {
        if ctx.is_static_analysis_enable() {
            ir.create_var(Eval::Val(U256::from(1024)))
        } else {
            ir.create_var(Eval::ArgsSize)
        }
    }
}

fn call_data_load(params: Vec<VarId>, ir: &mut Hir, ctx: &mut Context) -> ExecutionResult {
    let offset = params[0];
    if ctx.is_static_analysis_enable() {
        ctx.disable_static_analysis();
        if let Some(offset) = ir.resolve_var(offset) {
            if offset.is_zero() {
                let mut buf = [0u8; 32];
                buf[0..4].copy_from_slice(ctx.env().hash().as_ref().as_slice());
                let id = ir.create_var(Eval::Val(U256::from(buf)));
                return ExecutionResult::Output(vec![id]);
            }
        }
    }

    ExecutionResult::Output(vec![ir.create_var(Eval::Args(offset))])
}
