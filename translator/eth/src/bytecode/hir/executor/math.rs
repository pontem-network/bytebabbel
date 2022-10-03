use std::ops::{Div, Rem};

use evm_core::eval::arithmetic;
use evm_core::eval::bitwise;
use evm_core::utils::I256;
use primitive_types::U256;

use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir::ir::var::{Expr, VarId};
use crate::Hir;

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum UnaryOp {
    IsZero,
    Not,
}

impl UnaryOp {
    pub fn calc(&self, param: U256) -> U256 {
        match self {
            UnaryOp::IsZero => {
                if param.is_zero() {
                    U256::one()
                } else {
                    U256::zero()
                }
            }
            UnaryOp::Not => !param,
        }
    }
}

impl InstructionHandler for UnaryOp {
    fn handle(&self, params: Vec<VarId>, ir: &mut Hir, ctx: &mut Context) -> ExecutionResult {
        if !ctx.is_in_loop() {
            let param = ir.resolve_var(params[0]);
            if let Some(param) = param {
                let id = ir.create_var(Expr::Val(self.calc(param)));
                return ExecutionResult::Output(vec![id]);
            }
        }

        let expr = ir.var(&params[0]);
        let id = ir.create_var(Expr::UnaryOp(*self, Box::new(expr.clone())));
        ExecutionResult::Output(vec![id])
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum BinaryOp {
    Eq,
    Lt,
    Gt,
    Shr,
    Shl,
    Sar,
    Add,
    And,
    Or,
    Xor,
    Mul,
    Sub,
    Div,
    SDiv,
    SLt,
    SGt,
    Byte,
    Mod,
    SMod,
    Exp,
    SignExtend,
}

impl InstructionHandler for BinaryOp {
    fn handle(&self, params: Vec<VarId>, ir: &mut Hir, ctx: &mut Context) -> ExecutionResult {
        let a = params[0];
        let b = params[1];
        if !ctx.is_in_loop() {
            {
                let a = ir.resolve_var(a);
                let b = ir.resolve_var(b);
                if let (Some(a), Some(b)) = (a, b) {
                    let res = self.calc(a, b);
                    let id = ir.create_var(Expr::Val(res));
                    return ExecutionResult::Output(vec![id]);
                }
            }
            if self == &BinaryOp::Eq && a == b {
                let id = ir.create_var(Expr::Val(U256::one()));
                return ExecutionResult::Output(vec![id]);
            }
        }

        let id = ir.create_var(Expr::BinaryOp(
            *self,
            Box::new(Expr::Var(a)),
            Box::new(Expr::Var(b)),
        ));
        ExecutionResult::Output(vec![id])
    }
}

impl BinaryOp {
    pub fn calc(&self, mut a: U256, mut b: U256) -> U256 {
        match self {
            BinaryOp::Eq => {
                if a == b {
                    U256::one()
                } else {
                    U256::zero()
                }
            }
            BinaryOp::Lt => {
                if a < b {
                    U256::one()
                } else {
                    U256::zero()
                }
            }
            BinaryOp::Gt => {
                if a > b {
                    U256::one()
                } else {
                    U256::zero()
                }
            }
            BinaryOp::Shr => {
                if b == U256::zero() || a >= U256::from(256) {
                    U256::zero()
                } else {
                    b >> a.as_usize()
                }
            }
            BinaryOp::Shl => {
                if b == U256::zero() || a >= U256::from(256) {
                    U256::zero()
                } else {
                    b << a.as_usize()
                }
            }
            BinaryOp::Sar => bitwise::sar(a, b),
            BinaryOp::Add => a.overflowing_add(b).0,
            BinaryOp::And => a & b,
            BinaryOp::Or => a | b,
            BinaryOp::Xor => a ^ b,
            BinaryOp::Mul => a.overflowing_mul(b).0,
            BinaryOp::Sub => a.overflowing_sub(b).0,
            BinaryOp::Div => {
                if b == U256::zero() {
                    U256::zero()
                } else {
                    a / b
                }
            }
            BinaryOp::SDiv => U256::from(I256::from(a).div(I256::from(b))),
            BinaryOp::SLt => {
                if I256::from(a).lt(&I256::from(b)) {
                    U256::one()
                } else {
                    U256::zero()
                }
            }
            BinaryOp::SGt => {
                if I256::from(a).gt(&I256::from(b)) {
                    U256::one()
                } else {
                    U256::zero()
                }
            }
            BinaryOp::Byte => {
                let mut ret = U256::zero();

                for i in 0..256 {
                    if i < 8 && a < 32.into() {
                        let o: usize = a.as_usize();
                        let t = 255 - (7 - i + 8 * o);
                        let bit_mask = U256::one() << t;
                        let value = (b & bit_mask) >> t;
                        ret = ret.overflowing_add(value << i).0;
                    }
                }
                ret
            }
            BinaryOp::Mod => {
                if b == U256::zero() {
                    U256::zero()
                } else {
                    a.rem(b)
                }
            }
            BinaryOp::SMod => {
                if b == U256::zero() {
                    U256::zero()
                } else {
                    U256::from(I256::from(a).rem(I256::from(b)))
                }
            }
            BinaryOp::Exp => {
                let one = U256::one();
                let zero = U256::zero();
                let mut res = one;
                while b != zero {
                    if b & one != zero {
                        res = res.overflowing_mul(a).0;
                    }
                    b >>= 1;
                    a = a.overflowing_mul(a).0;
                }
                res
            }
            BinaryOp::SignExtend => {
                if a < U256::from(32) {
                    let bit_index = (8 * a.low_u32() + 7) as usize;
                    let bit = b.bit(bit_index);
                    let mask = (U256::one() << bit_index) - U256::one();
                    if bit {
                        b | !mask
                    } else {
                        b & mask
                    }
                } else {
                    b
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TernaryOp {
    AddMod,
    MulMod,
}

impl InstructionHandler for TernaryOp {
    fn handle(&self, params: Vec<VarId>, ir: &mut Hir, ctx: &mut Context) -> ExecutionResult {
        let op1 = params[0];
        let op2 = params[1];
        let op3 = params[2];

        if !ctx.is_in_loop() {
            let op1 = ir.resolve_var(op1);
            let op2 = ir.resolve_var(op2);
            let op3 = ir.resolve_var(op3);
            if let (Some(op1), Some(op2), Some(op3)) = (op1, op2, op3) {
                let res = self.calc(op1, op2, op3);
                let id = ir.create_var(Expr::Val(res));
                return ExecutionResult::Output(vec![id]);
            }
        }
        let op1 = ir.var(&op1).clone();
        let op2 = ir.var(&op2).clone();
        let op3 = ir.var(&op3).clone();

        let id = ir.create_var(Expr::TernaryOp(
            *self,
            Box::new(op1),
            Box::new(op2),
            Box::new(op3),
        ));
        ExecutionResult::Output(vec![id])
    }
}

impl TernaryOp {
    pub fn calc(&self, op1: U256, op2: U256, op3: U256) -> U256 {
        match self {
            TernaryOp::AddMod => arithmetic::addmod(op1, op2, op3),
            TernaryOp::MulMod => arithmetic::mulmod(op1, op2, op3),
        }
    }
}
