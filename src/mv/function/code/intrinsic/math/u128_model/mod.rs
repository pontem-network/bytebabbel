use crate::evm::bytecode::executor::types::U256;
use crate::mv::function::code::context::Context;
use crate::mv::function::code::intrinsic::math::{Literal, MathModel, PrepareSignatures};
use crate::mv::function::signature::SignatureWriter;
use move_binary_format::file_format::{Bytecode, SignatureToken};

mod binary_ops;
mod cast;
mod unary_ops;

#[derive(Default, Copy, Clone)]
pub struct U128MathModel;

impl MathModel for U128MathModel {
    fn math_type() -> SignatureToken {
        SignatureToken::U128
    }
}

impl Literal for U128MathModel {
    fn set_literal(&self, ctx: &mut Context, val: &U256) -> SignatureToken {
        if val > &U256::from(u128::MAX) {
            ctx.write_code(Bytecode::LdU128(u128::MAX));
        } else {
            ctx.write_code(Bytecode::LdU128(val.as_u128()));
        }
        SignatureToken::U128
    }
}

impl PrepareSignatures for U128MathModel {
    fn make_signature(&mut self, _: &mut SignatureWriter) {
        // no-op
    }
}
