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
    fn handle(&self, _: Vec<Rc<Expr>>, _: &mut Context) -> ExecutionResult {
        match self {
            TxMeta::Balance => U256::zero(),
            TxMeta::Origin => U256::zero(),
            TxMeta::Caller => {
                return ExecutionResult::from(Expr::Signer);
            }
            TxMeta::CallValue => U256::zero(),
            TxMeta::Blockhash => U256::zero(),
            TxMeta::Timestamp => U256::zero(),
            TxMeta::Difficulty => U256::zero(),
            TxMeta::Number => U256::zero(),
            TxMeta::GasPrice => U256::zero(),
            TxMeta::Coinbase => U256::zero(),
            TxMeta::GasLimit => U256::MAX,
            TxMeta::Gas => U256::MAX,
        }
        .into()
    }
}

pub struct CallDataSize;

impl InstructionHandler for CallDataSize {
    fn handle(&self, _: Vec<Rc<Expr>>, ctx: &mut Context) -> ExecutionResult {
        if ctx.flags().native_input {
            ctx.fun().call_data_size().into()
        } else if ctx.is_static_analysis_enable() {
            U256::from(1024).into()
        } else {
            Expr::ArgsSize
        }
        .into()
    }
}

pub struct CallDataLoad;

impl InstructionHandler for CallDataLoad {
    fn handle(&self, params: Vec<Rc<Expr>>, ctx: &mut Context) -> ExecutionResult {
        let offset = &params[0];
        let expr = if ctx.flags().native_input {
            if let Some(offset) = offset.resolve(ctx) {
                if offset.is_zero() {
                    Expr::Val(ctx.fun().hash().into())
                } else {
                    Expr::Val(U256::from(((offset.as_u128() - 4) / 32) + 1))
                }
            } else {
                panic!("unsupported dynamic types");
            }
        } else {
            if ctx.is_static_analysis_enable() {
                ctx.disable_static_analysis();
                if let Some(offset) = offset.resolve(ctx) {
                    if offset.is_zero() {
                        return ExecutionResult::Expr(vec![Rc::new(Expr::Val(
                            ctx.fun().hash().into(),
                        ))]);
                    }
                }
            }
            Expr::ArgsSize
        };
        ExecutionResult::Expr(vec![Rc::new(expr)])
    }
}
