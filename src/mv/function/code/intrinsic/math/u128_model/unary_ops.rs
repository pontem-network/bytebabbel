use crate::evm::bytecode::executor::ops::UnaryOp;
use crate::mv::function::code::intrinsic::math::{CastBool, UnaryOpCode};
use crate::mv::function::code::writer::CodeWriter;
use crate::U128MathModel;
use move_binary_format::file_format::{Bytecode, SignatureToken};

impl UnaryOpCode for U128MathModel {
    fn code(&self, code: &mut CodeWriter, op: UnaryOp, a: SignatureToken) -> SignatureToken {
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
                    code.extend([Bytecode::LdU128(0), Bytecode::Eq]);
                } else {
                    /*
                       if val {
                           false
                       } else {
                           true
                       }
                    */
                    code.push(Bytecode::Not);
                }
            }
            UnaryOp::Not => {
                if a == SignatureToken::U128 {
                    self.write_to_bool(code);
                }
                code.push(Bytecode::Not);
            }
        }
        SignatureToken::Bool
    }
}
