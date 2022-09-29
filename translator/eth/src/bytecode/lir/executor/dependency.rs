use crate::bytecode::hir::ir::var::VarId;
use crate::bytecode::lir::context::Context;
use crate::bytecode::lir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::lir::ir::{Expr, Lir};
use primitive_types::U256;

pub struct Sha3;

impl InstructionHandler for Sha3 {
    fn handle(&self, mut params: Vec<Expr>, ir: &mut Lir, ctx: &mut Context) -> ExecutionResult {
        let len = params.remove(1);
        let addr = params.remove(0);
        let id = ir.assign(ctx.next_var(), Expr::Hash(Box::new(addr), Box::new(len)));
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
        // let id = ir.create_var(Expr::Val(val));
        ExecutionResult::Output(Expr::Val(val))
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
    // let offset = params.remove(0);
    // if ctx.flags().native_input {
    //     if let Some(offset) = ir.resolve_var(offset) {
    //         if offset.is_zero() {
    //             ExecutionResult::Output(vec![ctx.fun().hash().as_frame().into()])
    //         } else {
    //             let index = ((offset - U256::from(4)) / U256::from(32)) + U256::one();
    //             ExecutionResult::Output(vec![
    //                 ir.create_var(Expr::Args(Box::new(VarId::from(index.as_u64()).into())))
    //             ])
    //         }
    //     } else {
    //         panic!("unsupported dynamic types");
    //     }
    // } else {
    //     if ctx.is_static_analysis_enable() {
    //         ctx.disable_static_analysis();
    //         if let Some(offset) = ir.resolve_var(offset) {
    //             if offset.is_zero() {
    //                 return ExecutionResult::Output(vec![ctx.fun().hash().as_frame().into()]);
    //             }
    //         }
    //     }
    //     ExecutionResult::Output(vec![Expr::Args(Box::new(offset))])
    // }
    todo!()
}
