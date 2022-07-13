use crate::mv::function::code::intrinsic::math::Cast;
use crate::mv::function::code::writer::CodeWriter;
use move_binary_format::file_format::{Bytecode, SignatureIndex, SignatureToken};

const EU128_OVERFLOW: u64 = 1;

pub struct U256Math {
    pub vec_sig_index: SignatureIndex,
}

impl Cast for U256Math {
    /// u128 -> u256 ([u64; 4])
    fn cast_from_u128(&self, code: &mut CodeWriter) {
        let input = code.set_var(SignatureToken::U128);
        code.extend([
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
        code.release_local(input);
    }

    /// u256([u64; 4]) -> u128   
    fn cast_to_u128(&self, code: &mut CodeWriter) {
        let tmp_u128 = code.borrow_local(SignatureToken::U128);
        let tmp_u64 = code.borrow_local(SignatureToken::U64);
        code.extend([
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
        let pc = code.pc();

        code.extend([
            Bytecode::BrTrue(pc + 3),
            Bytecode::LdU64(EU128_OVERFLOW),
            Bytecode::Abort,
            Bytecode::CopyLoc(tmp_u128),
        ]);
        code.release_local(tmp_u64);
        code.release_local(tmp_u128);
    }
}
