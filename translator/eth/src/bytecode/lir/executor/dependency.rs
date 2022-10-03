use crate::bytecode::lir::context::Context;
use crate::bytecode::lir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::lir::ir::{Expr, Lir};
use primitive_types::U256;

pub struct Sha3;

impl InstructionHandler for Sha3 {
    fn handle(&self, mut params: Vec<Expr>, ir: &mut Lir, ctx: &mut Context) -> ExecutionResult {
        let len = params.remove(1);
        let addr = params.remove(0);
        let id = ir.assign(Expr::Hash(Box::new(addr), Box::new(len)), ctx);
        ExecutionResult::Output(id.into())
    }
}

pub struct Address;

impl InstructionHandler for Address {
    fn handle(&self, _: Vec<Expr>, _: &mut Lir, ctx: &mut Context) -> ExecutionResult {
        ExecutionResult::Output(ctx.address().into())
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
    fn handle(&self, params: Vec<Expr>, ir: &mut Lir, ctx: &mut Context) -> ExecutionResult {
        let val = match self {
            TxMeta::Balance => U256::zero(),
            TxMeta::Origin => U256::zero(),
            TxMeta::Caller => {
                return ExecutionResult::Output(Expr::Signer);
            }
            TxMeta::CallValue => U256::zero(),
            TxMeta::CallDataLoad => {
                return call_data_load(params, ir, ctx);
            }
            TxMeta::CallDataSize => {
                return call_data_size(ctx);
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
        ExecutionResult::Output(val.into())
    }
}

fn call_data_size(ctx: &mut Context) -> ExecutionResult {
    let expr = if ctx.flags().native_input {
        ctx.fun().call_data_size().into()
    } else if ctx.is_static_analysis_enable() {
        U256::from(1024).into()
    } else {
        Expr::ArgsSize
    };
    ExecutionResult::Output(expr)
}

fn call_data_load(mut params: Vec<Expr>, ir: &mut Lir, ctx: &mut Context) -> ExecutionResult {
    let offset = params.remove(0);
    if ctx.flags().native_input {
        if let Some(offset) = offset.resolve(ir, ctx) {
            if offset.is_zero() {
                ExecutionResult::Output(ctx.fun().hash().as_frame().into())
            } else {
                let index = ((offset - U256::from(4)) / U256::from(32)) + U256::one();
                ExecutionResult::Output(Expr::Args(Box::new(index.into())))
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
        ExecutionResult::Output(Expr::Args(Box::new(offset)))
    }
}