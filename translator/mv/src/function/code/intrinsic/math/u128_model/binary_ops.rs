#![allow(dead_code)]

use crate::function::code::context::Context;
use crate::function::code::intrinsic::math::u128_model::U128MathModel;
use crate::function::code::intrinsic::math::{BinaryOpCode, CastBool, MathModel};
use evm::bytecode::executor::ops::BinaryOp;
use move_binary_format::file_format::{Bytecode, LocalIndex, SignatureToken};

impl BinaryOpCode for U128MathModel {
    fn code(
        &self,
        ctx: &mut Context,
        op: BinaryOp,
        a: SignatureToken,
        b: SignatureToken,
    ) -> SignatureToken {
        match op {
            BinaryOp::EQ => {
                self.cast_to_cmp(ctx, a, b);
                ctx.write_code(Bytecode::Eq);
                SignatureToken::Bool
            }
            BinaryOp::Lt => {
                self.cast_to_cmp(ctx, a, b);
                ctx.write_code(Bytecode::Lt);
                SignatureToken::Bool
            }
            BinaryOp::Gt => {
                self.cast_to_cmp(ctx, a, b);
                ctx.write_code(Bytecode::Gt);
                SignatureToken::Bool
            }
            BinaryOp::Shr => {
                self.cast(ctx, a, b);
                ctx.write_code(Bytecode::Shr);
                SignatureToken::U128
            }
            BinaryOp::Add => {
                self.cast(ctx, a, b);
                ctx.write_code(Bytecode::Add);
                SignatureToken::U128
            }
            BinaryOp::And => {
                self.cast(ctx, a, b);
                ctx.write_code(Bytecode::BitAnd);
                SignatureToken::U128
            }
            BinaryOp::Mul => {
                self.cast(ctx, a, b);
                ctx.write_code(Bytecode::Mul);
                SignatureToken::U128
            }
            BinaryOp::Sub => {
                self.cast(ctx, a, b);
                ctx.write_code(Bytecode::Sub);
                SignatureToken::U128
            }
            BinaryOp::Div => {
                self.cast(ctx, a, b);
                safe_div(ctx);
                SignatureToken::U128
            }
            BinaryOp::SLt => {
                todo!()
            }
            BinaryOp::Byte => {
                todo!()
            }
        }
    }
}

impl U128MathModel {
    fn cast_to_cmp(&self, code: &mut Context, a: SignatureToken, b: SignatureToken) {
        if a == SignatureToken::Bool || b == SignatureToken::Bool {
            if b != SignatureToken::Bool {
                self.write_to_bool(code);
            }
            let b_var = code.set_var(SignatureToken::Bool);
            if b != SignatureToken::Bool {
                self.write_to_bool(code);
            }
            code.move_local(b_var);
        }
    }

    fn cast(&self, code: &mut Context, a: SignatureToken, b: SignatureToken) {
        if a != SignatureToken::U128 || b != SignatureToken::U128 {
            if b == SignatureToken::Bool {
                self.write_from_bool(code);
            };
            let b_var = code.set_var(U128MathModel::math_type());

            if a == SignatureToken::Bool {
                self.write_from_bool(code);
            }
            code.move_local(b_var);
        }
    }

    fn cast_and_store_local(
        &self,
        code: &mut Context,
        a: SignatureToken,
        b: SignatureToken,
    ) -> (LocalIndex, LocalIndex) {
        if a != SignatureToken::U128 || b != SignatureToken::U128 {
            if b == SignatureToken::Bool {
                self.write_from_bool(code);
            };
            let var_b = code.set_var(U128MathModel::math_type());

            if a == SignatureToken::Bool {
                self.write_from_bool(code);
            }
            let var_a = code.set_var(U128MathModel::math_type());
            (var_b, var_a)
        } else {
            let var_b = code.set_var(U128MathModel::math_type());
            let var_a = code.set_var(U128MathModel::math_type());
            (var_b, var_a)
        }
    }
}

///
/// let revert_b = u128::max - b;
/// if revert_b < a {
///     /// overflow
///     a - revert_b - 1
/// } else {
///     a + b
/// }
///
fn overflowing_add(ctx: &mut Context, a: LocalIndex, b: LocalIndex) {
    ctx.extend_code([
        Bytecode::LdU128(u128::MAX),
        Bytecode::CopyLoc(b),
        Bytecode::Sub,
    ]);
    let revert_b = ctx.set_var(SignatureToken::U128);
    ctx.extend_code([
        Bytecode::CopyLoc(revert_b),
        Bytecode::CopyLoc(a),
        Bytecode::Lt,
    ]);
    let res = ctx.borrow_local(SignatureToken::U128);
    let pc = ctx.pc();
    ctx.extend_code([
        Bytecode::BrTrue(pc + 6),
        Bytecode::MoveLoc(a),
        Bytecode::MoveLoc(b),
        Bytecode::Add,
        Bytecode::StLoc(res),
    ]);
    let pc = ctx.pc();
    ctx.write_code(Bytecode::Branch(pc + 7));
    ctx.extend_code([
        Bytecode::MoveLoc(a),
        Bytecode::MoveLoc(revert_b),
        Bytecode::Sub,
        Bytecode::LdU128(1),
        Bytecode::Sub,
        Bytecode::StLoc(res),
    ]);
    ctx.move_local(res);
    ctx.release_local(revert_b);
}

///
/// if b > a {
/// // overflow
///  u128::MAX - (b - a) + 1
/// } else {
///  a - b
/// }
fn overflowing_sub(ctx: &mut Context, a: LocalIndex, b: LocalIndex) {
    ctx.extend_code([Bytecode::CopyLoc(a), Bytecode::CopyLoc(b), Bytecode::Gt]);
    let pc = ctx.pc();
    let res = ctx.borrow_local(SignatureToken::U128);
    ctx.extend_code([
        Bytecode::BrTrue(pc + 6),
        Bytecode::MoveLoc(b),
        Bytecode::MoveLoc(a),
        Bytecode::Sub,
        Bytecode::StLoc(res),
    ]);
    let pc = ctx.pc();
    ctx.extend_code([
        Bytecode::Branch(pc + 9),
        Bytecode::LdU128(u128::MAX),
        Bytecode::MoveLoc(b),
        Bytecode::MoveLoc(a),
        Bytecode::Sub,
        Bytecode::Sub,
        Bytecode::LdU128(1),
        Bytecode::Add,
        Bytecode::StLoc(res),
    ]);
    ctx.move_local(res)
}

///
/// if b == 0 {
///   0
/// } else {
///   a / b
/// }
fn safe_div(ctx: &mut Context) {
    let mut state = ctx.store_stack();
    let b = state.take_last();
    let a = state.take_last();

    ctx.extend_code([Bytecode::CopyLoc(b), Bytecode::LdU128(0), Bytecode::Eq]);
    let pc = ctx.pc();
    ctx.extend_code([
        Bytecode::BrTrue(pc + 5),
        Bytecode::MoveLoc(a),
        Bytecode::MoveLoc(b),
        Bytecode::Div,
        Bytecode::StLoc(b),
    ]);

    ctx.release_local(a);
    ctx.restore_stack(state);
    ctx.move_local(b);
}
