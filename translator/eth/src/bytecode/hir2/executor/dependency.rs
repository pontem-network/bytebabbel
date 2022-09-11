use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir2::ir::expression::Expr;
use crate::bytecode::hir2::ir::Hir2;
use primitive_types::U256;

pub struct Sha3;

impl InstructionHandler for Sha3 {
    fn handle(&self, mut params: Vec<Expr>, ir: &mut Hir2, _: &mut Context) -> ExecutionResult {
        let len = params.remove(1);
        let offset = params.remove(0);
        ExecutionResult::Output(vec![Expr::Hash {
            mem_offset: Box::new(offset),
            mem_len: Box::new(len),
        }])
    }
}

pub struct Address;

impl InstructionHandler for Address {
    fn handle(&self, _: Vec<Expr>, ir: &mut Hir2, ctx: &mut Context) -> ExecutionResult {
        ExecutionResult::Output(vec![Expr::Val(ctx.address())])
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
    fn handle(&self, mut params: Vec<Expr>, ir: &mut Hir2, ctx: &mut Context) -> ExecutionResult {
        let val = match self {
            TxMeta::Balance => U256::zero(),
            TxMeta::Origin => U256::zero(),
            TxMeta::Caller => {
                return ExecutionResult::Output(vec![Expr::Signer]);
            }
            TxMeta::CallValue => U256::zero(),
            TxMeta::CallDataLoad => {
                let offset = params.remove(0);
                if ctx.is_static_analysis_enable() {
                    let resolver_offset = offset.resolve(ctx);
                    ctx.disable_static_analysis();

                    if let Some(offset) = resolver_offset {
                        if offset.is_zero() {
                            let mut buf = [0u8; 32];
                            buf[0..4].copy_from_slice(ctx.env().hash().as_ref().as_slice());
                            return ExecutionResult::Output(vec![Expr::Val(U256::from(buf))]);
                        }
                    }
                }
                return ExecutionResult::Output(vec![Expr::Args {
                    args_offset: Box::new(offset),
                }]);
            }
            TxMeta::CallDataSize => {
                let expr = if ctx.is_static_analysis_enable() {
                    Expr::Val(U256::from(1024))
                } else {
                    Expr::ArgsSize
                };
                return ExecutionResult::Output(vec![expr]);
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
        ExecutionResult::Output(vec![Expr::Val(val)])
    }
}
