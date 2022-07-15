use crate::evm::bytecode::executor::ops::{BinaryOp, UnaryOp};
use crate::mv::function::code::intrinsic::math::{
    BinaryOpCode, CastBool, CastU128, MathModel, PrepareSignatures, Type, UnaryOpCode,
};
use crate::mv::function::code::writer::CodeWriter;
use crate::mv::function::signature::SignatureWriter;
use move_binary_format::file_format::SignatureToken;

#[derive(Default)]
pub struct U128MathModel;

impl CastU128 for U128MathModel {
    fn write_from_u128(&self, _bytecode: &mut CodeWriter) {
        todo!()
    }

    fn write_to_u128(&self, _bytecode: &mut CodeWriter) {
        todo!()
    }
}

impl CastBool for U128MathModel {
    fn write_from_bool(&self, _bytecode: &mut CodeWriter) {
        todo!()
    }

    fn write_to_bool(&self, _bytecode: &mut CodeWriter) {
        todo!()
    }
}

impl BinaryOpCode for U128MathModel {
    fn code(
        &self,
        _writer: &mut CodeWriter,
        _op: BinaryOp,
        _a: SignatureToken,
        _b: SignatureToken,
    ) -> Type {
        todo!()
    }
}

impl UnaryOpCode for U128MathModel {
    fn code(&self, _writer: &mut CodeWriter, _op: UnaryOp, _a: SignatureToken) -> Type {
        todo!()
    }
}

impl PrepareSignatures for U128MathModel {
    fn make_signature(&mut self, _sw: &mut SignatureWriter) {
        // no-op
    }
}

impl MathModel for U128MathModel {}
