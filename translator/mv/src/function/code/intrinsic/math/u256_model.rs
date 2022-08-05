use crate::function::code::context::Context;
use crate::function::code::intrinsic::math::{
    BinaryOpCode, CastBool, CastU128, Literal, MathModel, PrepareSignatures, UnaryOpCode,
};
use crate::function::signature::SignatureWriter;
use evm::bytecode::executor::ops::{BinaryOp, UnaryOp};
use evm::bytecode::types::U256;
use move_binary_format::file_format::{Bytecode, SignatureIndex, SignatureToken};

const EU128_OVERFLOW: u64 = 1;

#[derive(Default, Clone)]
pub struct U256MathModel {
    vec_sig_index: SignatureIndex,
}

impl CastU128 for U256MathModel {
    /// u128 -> u256 ([u64; 4])
    fn write_from_u128(&self, ctx: &mut Context) -> SignatureToken {
        let input = ctx.set_var(SignatureToken::U128);
        ctx.extend_code([
            Bytecode::LdU64(0),
            Bytecode::LdU64(0),
            Bytecode::CopyLoc(input),
            Bytecode::LdU8(64),
            Bytecode::Shr,
            Bytecode::CastU64,
            Bytecode::CopyLoc(input),
            Bytecode::LdU128(u64::MAX as u128),
            Bytecode::BitAnd,
            Bytecode::CastU64,
            Bytecode::VecPack(self.vec_sig_index, 4),
        ]);
        ctx.release_local(input);
        U256MathModel::math_type()
    }

    /// u256([u64; 4]) -> u128   
    fn write_to_u128(&self, ctx: &mut Context) {
        let tmp_u128 = ctx.borrow_local(SignatureToken::U128);
        let tmp_u64 = ctx.borrow_local(SignatureToken::U64);
        ctx.extend_code([
            Bytecode::VecUnpack(self.vec_sig_index, 4),
            Bytecode::CastU128,
            Bytecode::StLoc(tmp_u128),
            Bytecode::CastU128,
            Bytecode::LdU8(64),
            Bytecode::Shl,
            Bytecode::CopyLoc(tmp_u128),
            Bytecode::Add,
            Bytecode::StLoc(tmp_u128),
            Bytecode::StLoc(tmp_u64),
            Bytecode::Pop,
            Bytecode::CopyLoc(tmp_u64),
            Bytecode::LdU64(0),
            Bytecode::Eq,
        ]);
        let pc = ctx.pc();

        ctx.extend_code([
            Bytecode::BrTrue(pc + 3),
            Bytecode::LdU64(EU128_OVERFLOW),
            Bytecode::Abort,
            Bytecode::CopyLoc(tmp_u128),
        ]);
        ctx.release_local(tmp_u64);
        ctx.release_local(tmp_u128);
    }
}

impl CastBool for U256MathModel {
    fn write_from_bool(&self, _ctx: &mut Context) -> SignatureToken {
        U256MathModel::math_type()
    }

    fn write_to_bool(&self, _ctx: &mut Context) {
        todo!()
    }
}

impl BinaryOpCode for U256MathModel {
    fn code(
        &self,
        _ctx: &mut Context,
        _op: BinaryOp,
        _a: SignatureToken,
        _b: SignatureToken,
    ) -> SignatureToken {
        todo!()
    }
}

impl UnaryOpCode for U256MathModel {
    fn code(&self, _ctx: &mut Context, _op: UnaryOp, _a: SignatureToken) -> SignatureToken {
        todo!()
    }
}

impl PrepareSignatures for U256MathModel {
    fn make_signature(&mut self, sw: &mut SignatureWriter) {
        sw.make_signature(vec![SignatureToken::U64]);
    }
}

impl Literal for U256MathModel {
    fn set_literal(&self, _ctx: &mut Context, _val: &U256) -> SignatureToken {
        todo!()
    }
}

impl MathModel for U256MathModel {
    fn math_type() -> SignatureToken {
        SignatureToken::Vector(Box::new(SignatureToken::U64))
    }
}
