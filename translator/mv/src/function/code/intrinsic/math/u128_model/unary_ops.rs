use crate::function::code::context::Context;
use crate::function::code::intrinsic::math::u128_model::U128MathModel;
use crate::function::code::intrinsic::math::{CastBool, UnaryOpCode};
use evm::bytecode::executor::ops::UnaryOp;
use move_binary_format::file_format::{Bytecode, SignatureToken};

impl UnaryOpCode for U128MathModel {
    fn code(&self, ctx: &mut Context, op: UnaryOp, a: SignatureToken) -> SignatureToken {
        match op {
            UnaryOp::IsZero => {
                if a == SignatureToken::U128 {
                    /*
                    if val == 0 {
                        true
                    } else {
                        false
                    }
                     */
                    ctx.extend_code([Bytecode::LdU128(0), Bytecode::Eq]);
                } else {
                    /*
                       if val {
                           false
                       } else {
                           true
                       }
                    */
                    ctx.write_code(Bytecode::Not);
                }
            }
            UnaryOp::Not => {
                if a == SignatureToken::U128 {
                    self.write_to_bool(ctx);
                }
                ctx.write_code(Bytecode::Not);
            }
        }
        SignatureToken::Bool
    }
}