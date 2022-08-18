use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir::ir::var::{Var, VarId};
use crate::bytecode::types::{I256, U512};
use crate::{Hir, U256};
use std::ops::{Div, Rem};

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
                let id = ir.create_var(Var::Val(self.calc(param)));
                return ExecutionResult::Output(vec![id]);
            }
        }
        let id = ir.create_var(Var::UnaryOp(*self, params[0]));
        ExecutionResult::Output(vec![id])
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
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
    fn handle(&self, params: Vec<VarId>, ir: &mut Hir, ctx: &mut Context) -> ExecutionResult {
        let a = params[0];
        let b = params[1];
        if !ctx.is_in_loop() {
            {
                let a = ir.resolve_var(a);
                let b = ir.resolve_var(b);
                if let (Some(a), Some(b)) = (a, b) {
                    let res = self.calc(a, b);
                    let id = ir.create_var(Var::Val(res));
                    return ExecutionResult::Output(vec![id]);
                }
            }
            if self == &BinaryOp::EQ && a == b {
                let id = ir.create_var(Var::Val(U256::one()));
                return ExecutionResult::Output(vec![id]);
            }
        }

        let id = ir.create_var(Var::BinaryOp(*self, a, b));
        ExecutionResult::Output(vec![id])
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
            BinaryOp::Sar => {
                let value = I256::from(b);
                if value == I256::default() || a >= U256::from(256) {
                    match value.0 {
                        false => U256::zero(),
                        true => I256(true, U256::one()).into(),
                    }
                } else {
                    match value.0 {
                        false => value.1 >> a.as_usize(),
                        true => {
                            let shifted = ((value.1.overflowing_sub(U256::one()).0)
                                >> a.as_usize())
                            .overflowing_add(U256::one())
                            .0;
                            I256(true, shifted).into()
                        }
                    }
                }
            }
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

#[derive(Debug, Clone)]
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
                let id = ir.create_var(Var::Val(res));
                return ExecutionResult::Output(vec![id]);
            }
        }
        let id = ir.create_var(Var::TernaryOp(self.clone(), op1, op2, op3));
        ExecutionResult::Output(vec![id])
    }
}

impl TernaryOp {
    pub fn calc(&self, op1: U256, op2: U256, op3: U256) -> U256 {
        let op1 = U512::from(op1);
        let op2 = U512::from(op2);
        let op3 = U512::from(op3);
        match self {
            TernaryOp::AddMod => {
                if op3 == U512::zero() {
                    U256::zero()
                } else {
                    U256::from((op1 + op2) % op3)
                }
            }
            TernaryOp::MulMod => {
                if op3 == U512::zero() {
                    U256::zero()
                } else {
                    U256::from((op1 * op2) % op3)
                }
            }
        }
    }
}
