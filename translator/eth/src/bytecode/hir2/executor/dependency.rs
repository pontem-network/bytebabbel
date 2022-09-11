use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir2::ir::expression::Expr;
use primitive_types::U256;
use std::rc::Rc;

pub struct Sha3;

impl InstructionHandler for Sha3 {
    fn handle(&self, mut params: Vec<Rc<Expr>>, _: &mut Context) -> ExecutionResult {
        let len = params.remove(1);
        let offset = params.remove(0);
        ExecutionResult::Expr(vec![Rc::new(Expr::Hash {
            mem_offset: offset,
            mem_len: len,
        })])
    }
}

pub struct Address;

impl InstructionHandler for Address {
    fn handle(&self, _: Vec<Rc<Expr>>, ctx: &mut Context) -> ExecutionResult {
        ExecutionResult::Expr(vec![Rc::new(Expr::Val(ctx.address()))])
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
    fn handle(&self, mut params: Vec<Rc<Expr>>, ctx: &mut Context) -> ExecutionResult {
        let val = match self {
            TxMeta::Balance => U256::zero(),
            TxMeta::Origin => U256::zero(),
            TxMeta::Caller => {
                return ExecutionResult::Expr(vec![Rc::new(Expr::Signer)]);
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
                            return ExecutionResult::Expr(vec![Rc::new(Expr::Val(U256::from(
                                buf,
                            )))]);
                        }
                    }
                }
                return ExecutionResult::Expr(vec![Rc::new(Expr::Args {
                    args_offset: offset,
                })]);
            }
            TxMeta::CallDataSize => {
                let expr = if ctx.is_static_analysis_enable() {
                    Expr::Val(U256::from(1024))
                } else {
                    Expr::ArgsSize
                };
                return ExecutionResult::Expr(vec![Rc::new(expr)]);
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
        ExecutionResult::Expr(vec![Rc::new(Expr::Val(val))])
    }
}
