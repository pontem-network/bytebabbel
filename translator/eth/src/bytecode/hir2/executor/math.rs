use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir2::ir::expression::Expr;
use crate::bytecode::hir2::ir::Hir2;
use evm_core::eval::arithmetic;
use evm_core::eval::bitwise;
use evm_core::utils::I256;
use primitive_types::U256;
use std::ops::{Div, Rem};

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy, Ord, PartialOrd)]
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
    fn handle(&self, mut params: Vec<Expr>, ir: &mut Hir2, ctx: &mut Context) -> ExecutionResult {
        let param = params.remove(0);
        if !ctx.is_in_loop() {
            if let Some(param) = param.resolve(ctx) {
                return ExecutionResult::Output(vec![Expr::Val(self.calc(param))]);
            }
        }
        ExecutionResult::Output(vec![Expr::UnaryOp(*self, Box::new(param))])
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone, Ord, PartialOrd)]
pub enum BinaryOp {
    EQ,
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
    fn handle(&self, mut params: Vec<Expr>, ir: &mut Hir2, ctx: &mut Context) -> ExecutionResult {
        let b = params.remove(1);
        let a = params.remove(0);
        if !ctx.is_in_loop() {
            {
                let a = a.resolve(ctx);
                let b = b.resolve(ctx);
                if let (Some(a), Some(b)) = (a, b) {
                    let res = self.calc(a, b);
                    return ExecutionResult::Output(vec![Expr::Val(res)]);
                }
            }
            if self == &BinaryOp::EQ && a == b {
                return ExecutionResult::Output(vec![Expr::Val(U256::one())]);
            }
        }
        ExecutionResult::Output(vec![Expr::BinaryOp(*self, Box::new(a), Box::new(b))])
    }
}

impl BinaryOp {
    pub fn calc(&self, mut a: U256, mut b: U256) -> U256 {
        match self {
            BinaryOp::EQ => {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum TernaryOp {
    AddMod,
    MulMod,
}

impl InstructionHandler for TernaryOp {
    fn handle(&self, mut params: Vec<Expr>, ir: &mut Hir2, ctx: &mut Context) -> ExecutionResult {
        let op3 = params.remove(2);
        let op2 = params.remove(1);
        let op1 = params.remove(0);

        if !ctx.is_in_loop() {
            let op1 = op1.resolve(ctx);
            let op2 = op2.resolve(ctx);
            let op3 = op3.resolve(ctx);
            if let (Some(op1), Some(op2), Some(op3)) = (op1, op2, op3) {
                let res = self.calc(op1, op2, op3);
                return ExecutionResult::Output(vec![Expr::Val(res)]);
            }
        }
        let expr = Expr::TernaryOp(self.clone(), Box::new(op1), Box::new(op2), Box::new(op3));
        ExecutionResult::Output(vec![expr])
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
