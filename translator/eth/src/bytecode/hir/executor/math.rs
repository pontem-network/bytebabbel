use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir::ir::expression::Expr;
use evm_core::eval::arithmetic;
use evm_core::eval::bitwise;
use evm_core::utils::I256;
use primitive_types::U256;
use std::fmt::{Debug, Display, Formatter};
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
    fn handle(&self, mut params: Vec<Expr>, ctx: &mut Context) -> ExecutionResult {
        let param = params.remove(0);
        if !ctx.is_in_loop() {
            if let Some(param) = param.resolve(Some(ctx)) {
                return self.calc(param).into();
            }
        }
        Expr::UnaryOp(*self, Box::new(param)).into()
    }
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOp::IsZero => write!(f, "0 == "),
            UnaryOp::Not => write!(f, "!"),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone, Ord, PartialOrd)]
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
    fn handle(&self, mut params: Vec<Expr>, ctx: &mut Context) -> ExecutionResult {
        let b = params.remove(1);
        let a = params.remove(0);
        if !ctx.is_in_loop() {
            {
                let a = a.resolve(Some(ctx));
                let b = b.resolve(Some(ctx));
                if let (Some(a), Some(b)) = (a, b) {
                    let res = self.calc(a, b);
                    return res.into();
                }
            }
            if self == &BinaryOp::Eq && a == b {
                return U256::one().into();
            }
        }
        Expr::BinaryOp(*self, Box::new(a), Box::new(b)).into()
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

impl Display for BinaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOp::Eq => write!(f, "=="),
            BinaryOp::Lt => write!(f, "<"),
            BinaryOp::Gt => write!(f, ">"),
            BinaryOp::Shr => write!(f, ">>"),
            BinaryOp::Shl => write!(f, "<<"),
            BinaryOp::Sar => write!(f, ">>>"),
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::And => write!(f, "&"),
            BinaryOp::Or => write!(f, "|"),
            BinaryOp::Xor => write!(f, "^"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::Div => write!(f, "/"),
            BinaryOp::SDiv => write!(f, "//"),
            BinaryOp::SLt => write!(f, "<"),
            BinaryOp::SGt => write!(f, ">"),
            BinaryOp::Byte => write!(f, "#"),
            BinaryOp::Mod => write!(f, "%"),
            BinaryOp::SMod => write!(f, "%%"),
            BinaryOp::Exp => write!(f, "**"),
            BinaryOp::SignExtend => write!(f, "***"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum TernaryOp {
    AddMod,
    MulMod,
}

impl InstructionHandler for TernaryOp {
    fn handle(&self, mut params: Vec<Expr>, ctx: &mut Context) -> ExecutionResult {
        let op3 = params.remove(2);
        let op2 = params.remove(1);
        let op1 = params.remove(0);

        if !ctx.is_in_loop() {
            let op1 = op1.resolve(Some(ctx));
            let op2 = op2.resolve(Some(ctx));
            let op3 = op3.resolve(Some(ctx));
            if let (Some(op1), Some(op2), Some(op3)) = (op1, op2, op3) {
                let res = self.calc(op1, op2, op3);
                return res.into();
            }
        }

        Expr::TernaryOp(*self, Box::new(op1), Box::new(op2), Box::new(op3)).into()
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

impl Display for TernaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TernaryOp::AddMod => {
                write!(f, "addmod")
            }
            TernaryOp::MulMod => {
                write!(f, "mulmod")
            }
        }
    }
}
