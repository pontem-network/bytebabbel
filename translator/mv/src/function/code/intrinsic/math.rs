use crate::function::code::context::Context;
use crate::function::signature::SignatureWriter;
use evm::bytecode::executor::ops::{BinaryOp, UnaryOp};
use evm::bytecode::executor::types::U256;
use move_binary_format::file_format::SignatureToken;

pub mod u128_model;
pub mod u256_model;

pub trait MathModel:
    CastU128 + CastBool + BinaryOpCode + UnaryOpCode + PrepareSignatures + Literal + Clone
{
    fn math_type() -> SignatureToken;
}

pub trait CastU128 {
    fn write_from_u128(&self, bytecode: &mut Context) -> SignatureToken;
    fn write_to_u128(&self, bytecode: &mut Context);
}

pub trait CastBool {
    fn write_from_bool(&self, bytecode: &mut Context) -> SignatureToken;
    fn write_to_bool(&self, bytecode: &mut Context);
}

pub trait PrepareSignatures {
    fn make_signature(&mut self, sw: &mut SignatureWriter);
}

pub trait Literal {
    fn set_literal(&self, bytecode: &mut Context, val: &U256) -> SignatureToken;
}

pub trait BinaryOpCode {
    fn code(
        &self,
        writer: &mut Context,
        op: BinaryOp,
        a: SignatureToken,
        b: SignatureToken,
    ) -> SignatureToken;
}

pub trait UnaryOpCode {
    fn code(&self, writer: &mut Context, op: UnaryOp, a: SignatureToken) -> SignatureToken;
}