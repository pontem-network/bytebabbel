use crate::bytecode::llir::context::Context;
use crate::bytecode::llir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::llir::ir::var::{Var, VarId};
use crate::bytecode::llir::stack::FRAME_SIZE;
use crate::{Ir, U256};

pub struct Sha3;

impl InstructionHandler for Sha3 {
    fn handle(&self, _: Vec<VarId>, _: &mut Ir, _: &mut Context) -> ExecutionResult {
        todo!()
    }
}

pub struct Address;

impl InstructionHandler for Address {
    fn handle(&self, _: Vec<VarId>, ir: &mut Ir, ctx: &mut Context) -> ExecutionResult {
        let id = ir.create_var(Var::Val(ctx.address()));
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
    fn handle(&self, params: Vec<VarId>, ir: &mut Ir, context: &mut Context) -> ExecutionResult {
        let val = match self {
            TxMeta::Balance => todo!(),
            TxMeta::Origin => todo!(),
            TxMeta::Caller => todo!(),
            TxMeta::CallValue => todo!(),
            TxMeta::CallDataLoad => {
                if let Some(offset) = ir.resolve_var(params[0]) {
                    if offset.is_zero() {
                        let mut buf = [0u8; 32];
                        buf[0..4].copy_from_slice(context.env().hash().as_ref().as_slice());
                        U256::from(buf)
                    } else {
                        let index = (offset.as_usize() - 4) / FRAME_SIZE;
                        let id = ir.create_var(Var::Param(index as u16));
                        return ExecutionResult::Output(vec![id]);
                    }
                } else {
                    panic!("Unsupported dynamic call data load");
                }
            }
            TxMeta::CallDataSize => context.env().call_data_size(),
            TxMeta::Blockhash => U256::zero(),
            TxMeta::Timestamp => U256::zero(),
            TxMeta::Difficulty => U256::zero(),
            TxMeta::Number => U256::zero(),
            TxMeta::GasPrice => U256::zero(),
            TxMeta::Coinbase => U256::zero(),
            TxMeta::GasLimit => U256::MAX,
            TxMeta::Gas => U256::MAX,
        };
        let id = ir.create_var(Var::Val(val));
        ExecutionResult::Output(vec![id])
    }
}
