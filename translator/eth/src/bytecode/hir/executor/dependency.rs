use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir::ir::{Expr, _Expr};
use crate::Hir;
use primitive_types::U256;

pub struct Sha3;

impl InstructionHandler for Sha3 {
    fn handle(&self, mut params: Vec<Expr>, ir: &mut Hir, ctx: &mut Context) -> ExecutionResult {
        let len = params.remove(1);
        let addr = params.remove(0);
        let id = ir.assign(
            ctx.loc.wrap(_Expr::Hash(Box::new(addr), Box::new(len))),
            &mut ctx.vars,
        );
        ExecutionResult::Output(id.into())
    }
}

pub struct Address;

impl InstructionHandler for Address {
    fn handle(&self, _: Vec<Expr>, _: &mut Hir, ctx: &mut Context) -> ExecutionResult {
        ExecutionResult::Output(ctx.address().into())
    }
}

pub enum TxMeta {
    Origin,
    Caller,
    CallValue,
    CallDataLoad,
    CallDataSize,
    Coinbase,
    Difficulty,
    Balance,
    // @todo
    Gas,
    GasPrice,
    GasLimit,
    BlockHeight,
    BlockTimestamp,
    // @todo
    Blockhash,
}

impl InstructionHandler for TxMeta {
    fn handle(&self, params: Vec<Expr>, ir: &mut Hir, ctx: &mut Context) -> ExecutionResult {
        let val = match self {
            TxMeta::Origin => return ExecutionResult::Output(_Expr::Signer),
            TxMeta::Caller => {
                return ExecutionResult::Output(_Expr::Signer);
            }
            TxMeta::CallValue => U256::zero(),
            TxMeta::CallDataLoad => {
                return call_data_load(params, ir, ctx);
            }
            TxMeta::CallDataSize => {
                return call_data_size(ctx);
            }
            TxMeta::Difficulty => U256::zero(),
            TxMeta::Coinbase => U256::zero(),
            TxMeta::Balance => {
                if let Some(addr) = params.get(0) {
                    let addr = addr.unvar(ctx);
                    return ExecutionResult::Output(_Expr::Balance(Box::new(addr)));
                } else {
                    U256::zero()
                }
            }
            TxMeta::GasPrice => {
                return ExecutionResult::Output(_Expr::GasPrice);
            }
            TxMeta::GasLimit => {
                return ExecutionResult::Output(_Expr::GasLimit);
            }
            TxMeta::BlockTimestamp => {
                return ExecutionResult::Output(_Expr::BlockTimestamp);
            }
            TxMeta::BlockHeight => {
                return ExecutionResult::Output(_Expr::BlockHeight);
            }
            // @todo
            TxMeta::Gas => U256::MAX,
            // @todo
            TxMeta::Blockhash => U256::zero(),
        };
        ExecutionResult::Output(val.into())
    }
}

fn call_data_size(ctx: &mut Context) -> ExecutionResult {
    let expr = if ctx.flags().native_input {
        ctx.fun().call_data_size().into()
    } else if ctx.is_static_analysis_enable() {
        U256::from(1024).into()
    } else {
        _Expr::ArgsSize
    };
    ExecutionResult::Output(expr)
}

fn call_data_load(mut params: Vec<Expr>, ir: &mut Hir, ctx: &mut Context) -> ExecutionResult {
    let offset = params.remove(0);
    if ctx.flags().native_input {
        if let Some(offset) = offset.resolve(ir, ctx) {
            if offset.is_zero() {
                ExecutionResult::Output(ctx.fun().hash().as_frame().into())
            } else {
                let index = ((offset - U256::from(4)) / U256::from(32)) + U256::one();
                ExecutionResult::Output(_Expr::Args(Box::new(ctx.loc.wrap(index.into()))))
            }
        } else {
            panic!("unsupported dynamic types");
        }
    } else {
        if ctx.is_static_analysis_enable() {
            ctx.disable_static_analysis();
            if let Some(offset) = offset.resolve(ir, ctx) {
                if offset.is_zero() {
                    return ExecutionResult::Output(ctx.fun().hash().as_frame().into());
                }
            }
        }
        ExecutionResult::Output(_Expr::Args(Box::new(offset)))
    }
}
