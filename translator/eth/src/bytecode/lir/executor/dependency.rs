use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::ir::var::VarId;
use crate::bytecode::lir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::lir::ir::{Expr, Lir};
use primitive_types::U256;

pub struct Sha3;

impl InstructionHandler for Sha3 {
    fn handle(&self, params: Vec<Expr>, ir: &mut Lir, _: &mut Context) -> ExecutionResult {
        // let id = ir.create_var(Expr::Hash(
        //     Box::new(params[0].clone()),
        //     Box::new(params[1].clone()),
        // ));
        ExecutionResult::Output(vec![Expr::Hash(
            Box::new(params[0].clone()),
            Box::new(params[1].clone()),
        )])
    }
}

pub struct Address;

impl InstructionHandler for Address {
    fn handle(&self, _: Vec<Expr>, ir: &mut Lir, ctx: &mut Context) -> ExecutionResult {
        // let id = ir.create_var(Expr::Val(ctx.address()));
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
    fn handle(&self, params: Vec<Expr>, ir: &mut Lir, ctx: &mut Context) -> ExecutionResult {
        // let val = match self {
        //     TxMeta::Balance => U256::zero(),
        //     TxMeta::Origin => U256::zero(),
        //     TxMeta::Caller => {
        //         let id = ir.create_var(Expr::Signer);
        //         return ExecutionResult::Output(vec![id]);
        //     }
        //     TxMeta::CallValue => U256::zero(),
        //     TxMeta::CallDataLoad => {
        //         return call_data_load(params, ir, ctx);
        //     }
        //     TxMeta::CallDataSize => {
        //         return call_data_size(ir, ctx);
        //     }
        //     TxMeta::Blockhash => U256::zero(),
        //     TxMeta::Timestamp => U256::zero(),
        //     TxMeta::Difficulty => U256::zero(),
        //     TxMeta::Number => U256::zero(),
        //     TxMeta::GasPrice => U256::zero(),
        //     TxMeta::Coinbase => U256::zero(),
        //     TxMeta::GasLimit => U256::MAX,
        //     TxMeta::Gas => U256::MAX,
        // };
        // // let id = ir.create_var(Expr::Val(val));
        // ExecutionResult::Output(vec![Expr::Val(val)])
        todo!()
    }
}
//
// fn call_data_size(ir: &mut Lir, ctx: &mut Context) -> ExecutionResult {
//     let id = if ctx.flags().native_input {
//         ir.create_var(Expr::Val(ctx.fun().call_data_size()))
//     } else if ctx.is_static_analysis_enable() {
//         ir.create_var(Expr::Val(U256::from(1024)))
//     } else {
//         ir.create_var(Expr::ArgsSize)
//     };
//     ExecutionResult::Output(vec![id])
// }
//
// fn call_data_load(params: Vec<Expr>, ir: &mut Lir, ctx: &mut Context) -> ExecutionResult {
//     let offset = params[0].clone();
//     if ctx.flags().native_input {
//         if let Some(offset) = ir.resolve_var(offset) {
//             if offset.is_zero() {
//                 let id = ir.create_var(Expr::Val(ctx.fun().hash().as_frame()));
//                 ExecutionResult::Output(vec![id])
//             } else {
//                 let index = ((offset - U256::from(4)) / U256::from(32)) + U256::one();
//                 ExecutionResult::Output(vec![
//                     ir.create_var(Expr::Args(Box::new(VarId::from(index.as_u64()))))
//                 ])
//             }
//         } else {
//             panic!("unsupported dynamic types");
//         }
//     } else {
//         if ctx.is_static_analysis_enable() {
//             ctx.disable_static_analysis();
//             if let Some(offset) = ir.resolve_var(offset) {
//                 if offset.is_zero() {
//                     let id = ir.create_var(Expr::Val(ctx.fun().hash().as_frame()));
//                     return ExecutionResult::Output(vec![id]);
//                 }
//             }
//         }
//         ExecutionResult::Output(vec![ir.create_var(Expr::Args(offset))])
//     }
// }
