use crate::mv::function::code::context::Context;
use crate::mv::function::code::intrinsic::math::{CastBool, CastU128, MathModel};
use crate::U128MathModel;
use move_binary_format::file_format::{Bytecode, SignatureToken};

impl CastBool for U128MathModel {
    fn write_from_bool(&self, ctx: &mut Context) -> SignatureToken {
        let cmd = ctx.set_var(SignatureToken::Bool);
        let stack = ctx.store_stack();
        let tmp_var = ctx.borrow_local(SignatureToken::U128);
        ctx.move_local(cmd);

        let pc = ctx.pc();
        ctx.extend_code([
            Bytecode::BrTrue(pc + 4),
            Bytecode::LdU128(0),
            Bytecode::StLoc(tmp_var),
            Bytecode::Branch(pc + 6),
            Bytecode::LdU128(1),
            Bytecode::StLoc(tmp_var),
        ]);
        ctx.restore_stack(stack);
        ctx.move_local(tmp_var);
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
