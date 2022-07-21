use crate::mv::function::code::intrinsic::math::{CastBool, CastU128, MathModel};
use crate::U128MathModel;
use move_binary_format::file_format::{Bytecode, SignatureToken};
use crate::mv::function::code::context::Context;

impl CastBool for U128MathModel {
    fn write_from_bool(&self, ctx: &mut Context) -> SignatureToken {
        println!("write_from_bool");
        let pc = ctx.pc();
        let tmp_var = ctx.borrow_local(SignatureToken::U128);
        ctx.extend_code([
            Bytecode::BrTrue(pc + 4),
            Bytecode::LdU128(0),
            Bytecode::StLoc(tmp_var),
            Bytecode::Branch(pc + 5),
            Bytecode::LdU128(1),
            Bytecode::StLoc(tmp_var),
            Bytecode::MoveLoc(tmp_var),
        ]);
        ctx.release_local(tmp_var);
        U128MathModel::math_type()
    }

    fn write_to_bool(&self, bytecode: &mut Context) {
        bytecode.extend_code([Bytecode::LdU128(0), Bytecode::Eq, Bytecode::Not]);
    }
}

impl CastU128 for U128MathModel {
    fn write_from_u128(&self, _bytecode: &mut Context) -> SignatureToken {
        U128MathModel::math_type()
    }

    fn write_to_u128(&self, _bytecode: &mut Context) {}
}
