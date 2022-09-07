use crate::bytecode::hir::executor::math::BinaryOp;
use crate::bytecode::hir::ir::var::VarId;
use crate::bytecode::mir::ir::expression::{Expression, StackOpsBuilder};
use crate::bytecode::mir::ir::math::Operation;
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::{SType, Value};
use crate::bytecode::mir::translation::Variable;
use crate::MirTranslator;
use anyhow::Error;

impl<'a> MirTranslator<'a> {
    pub(super) fn translate_binary_op(
        &mut self,
        cmd: BinaryOp,
        op: VarId,
        op1: VarId,
        result: VarId,
    ) -> Result<(), Error> {
        let op = self.get_var(op)?;
        let op1 = self.get_var(op1)?;

        match cmd {
            BinaryOp::EQ => {
                let result = self.map_var(result, SType::Bool);
                translate_eq(self, op, op1, result)?;
            }
            BinaryOp::Lt => {
                let op = self.cast_number(op)?;
                let op1 = self.cast_number(op1)?;
                let result = self.map_var(result, SType::Bool);
                self.mir
                    .add_statement(Statement::CreateVar(result, Operation::Lt.expr(op, op1)));
            }
            BinaryOp::Gt => {
                let op = self.cast_number(op)?;
                let op1 = self.cast_number(op1)?;
                let result = self.map_var(result, SType::Bool);
                self.mir
                    .add_statement(Statement::CreateVar(result, Operation::Gt.expr(op, op1)));
            }
            BinaryOp::Shr => {
                translate_shr(self, op, op1, result)?;
            }
            BinaryOp::Shl => {
                translate_shl(self, op, op1, result)?;
            }
            BinaryOp::Sar => {
                todo!()
            }
            BinaryOp::Add => {
                translate_add(self, op, op1, result)?;
            }
            BinaryOp::And => {
                plain_u128_ops(self, Operation::BitAnd, op, op1, result)?;
            }
            BinaryOp::Or => {
                plain_u128_ops(self, Operation::BitOr, op, op1, result)?;
            }
            BinaryOp::Xor => {
                plain_u128_ops(self, Operation::Xor, op, op1, result)?;
            }
            BinaryOp::Mul => {
                // todo overflowing mul
                plain_u128_ops(self, Operation::Mul, op, op1, result)?;
            }
            BinaryOp::Sub => {
                translate_sub(self, op, op1, result)?;
            }
            BinaryOp::Div => {
                translate_div(self, op, op1, result)?;
            }
            BinaryOp::SDiv => {
                todo!()
            }
            BinaryOp::SLt => {
                translate_slt(self, op, op1, result)?;
            }
            BinaryOp::SGt => {
                todo!()
            }
            BinaryOp::Byte => {
                todo!()
            }
            BinaryOp::Mod => {
                translate_mod(self, op, op1, result)?;
            }
            BinaryOp::SMod => {
                todo!()
            }
            BinaryOp::Exp => {
                todo!()
            }
            BinaryOp::SignExtend => {
                todo!()
            }
        }

        Ok(())
    }
}

fn translate_shr(
    translator: &mut MirTranslator,
    op: Variable,
    op1: Variable,
    result: VarId,
) -> Result<(), Error> {
    // if op1 == 0 || op >= 256 {
    //     0
    // } else {
    //     op1 >> op
    // }
    let result = translator.map_var(result, SType::Number);

    let op = translator.cast_number(op)?;
    let op1 = translator.cast_number(op1)?;

    let cnd = StackOpsBuilder::default()
        .push_var(op1)
        .push_const(Value::U128(0))
        .binary_op(Operation::Eq, SType::Number, SType::Bool)?
        .push_var(op)
        .push_const(Value::U128(256))
        .binary_op(Operation::Ge, SType::Number, SType::Bool)?
        .binary_op(Operation::Or, SType::Bool, SType::Bool)?
        .build(SType::Bool)?;

    translator.mir.add_statement(Statement::IF {
        cnd,
        true_br: vec![Statement::CreateVar(
            result,
            Expression::Const(Value::U128(0)),
        )],
        false_br: vec![Statement::CreateVar(result, Operation::Shr.expr(op1, op))],
    });
    Ok(())
}
fn translate_shl(
    translator: &mut MirTranslator,
    op: Variable,
    op1: Variable,
    result: VarId,
) -> Result<(), Error> {
    // if op1 == 0 || op >= 256 {
    //     0
    // } else {
    //     op1 << op
    // }
    let result = translator.map_var(result, SType::Number);

    let op = translator.cast_number(op)?;
    let op1 = translator.cast_number(op1)?;

    let cnd = StackOpsBuilder::default()
        .push_var(op1)
        .push_const(Value::U128(0))
        .binary_op(Operation::Eq, SType::Number, SType::Bool)?
        .push_var(op)
        .push_const(Value::U128(256))
        .binary_op(Operation::Ge, SType::Number, SType::Bool)?
        .binary_op(Operation::Or, SType::Bool, SType::Bool)?
        .build(SType::Bool)?;

    translator.mir.add_statement(Statement::IF {
        cnd,
        true_br: vec![Statement::CreateVar(
            result,
            Expression::Const(Value::U128(0)),
        )],
        false_br: vec![Statement::CreateVar(result, Operation::Shl.expr(op1, op))],
    });
    Ok(())
}

fn translate_slt(
    translator: &mut MirTranslator,
    op: Variable,
    op1: Variable,
    result: VarId,
) -> Result<(), Error> {
    // todo signed comparison
    let op = translator.cast_number(op)?;
    let op1 = translator.cast_number(op1)?;
    let result = translator.map_var(result, SType::Bool);
    translator
        .mir
        .add_statement(Statement::CreateVar(result, Operation::Lt.expr(op, op1)));
    Ok(())
}

fn translate_div(
    translator: &mut MirTranslator,
    op: Variable,
    op1: Variable,
    result: VarId,
) -> Result<(), Error> {
    /*
        fn div(op: u256, op1: u256) -> u256 {
            if op1 == U256::zero() {
                U256::zero()
            } else {
                op / op1
            }
        }
    */
    let result = translator.map_var(result, SType::Number);

    let op = translator.cast_number(op)?;
    let op1 = translator.cast_number(op1)?;

    let cnd = StackOpsBuilder::default()
        .push_var(op1)
        .push_const(Value::U128(0))
        .binary_op(Operation::Eq, SType::Number, SType::Bool)?
        .build(SType::Bool)?;

    translator.mir.add_statement(Statement::IF {
        cnd,
        true_br: vec![Statement::CreateVar(
            result,
            Expression::Const(Value::U128(0)),
        )],
        false_br: vec![Statement::CreateVar(result, Operation::Div.expr(op, op1))],
    });

    Ok(())
}

fn translate_mod(
    translator: &mut MirTranslator,
    op: Variable,
    op1: Variable,
    result: VarId,
) -> Result<(), Error> {
    /*
        fn div(op: u256, op1: u256) -> u256 {
            if op1 == 0 {
                0
            } else {
                op % op1
            }
        }
    */
    let result = translator.map_var(result, SType::Number);

    let op = translator.cast_number(op)?;
    let op1 = translator.cast_number(op1)?;

    let cnd = StackOpsBuilder::default()
        .push_var(op1)
        .push_const(Value::U128(0))
        .binary_op(Operation::Eq, SType::Number, SType::Bool)?
        .build(SType::Bool)?;

    translator.mir.add_statement(Statement::IF {
        cnd,
        true_br: vec![Statement::CreateVar(
            result,
            Expression::Const(Value::U128(0)),
        )],
        false_br: vec![Statement::CreateVar(result, Operation::Mod.expr(op, op1))],
    });

    Ok(())
}

fn translate_eq(
    translator: &mut MirTranslator,
    op: Variable,
    op1: Variable,
    result: Variable,
) -> Result<(), Error> {
    //todo optimize qe cases with consts
    let (op, op1) = if op.s_type() == op1.s_type() {
        (op, op1)
    } else {
        let op = translator.cast_number(op)?;
        let op1 = translator.cast_number(op1)?;
        (op, op1)
    };
    translator
        .mir
        .add_statement(Statement::CreateVar(result, Operation::Eq.expr(op, op1)));
    Ok(())
}

/// if op1 > op {
///     // overflow u128::MAX - (op1 - op) + 1
/// } else {
///     op - op1
/// }
fn translate_sub(
    translator: &mut MirTranslator,
    op: Variable,
    op1: Variable,
    result: VarId,
) -> Result<(), Error> {
    let result = translator.map_var(result, SType::Number);

    let op = translator.cast_number(op)?;
    let op1 = translator.cast_number(op1)?;

    let cnd = StackOpsBuilder::default()
        .push_var(op1)
        .push_var(op)
        .binary_op(Operation::Gt, SType::Number, SType::Bool)?
        .build(SType::Bool)?;

    let true_br = StackOpsBuilder::default()
        .push_const(Value::U128(u128::MAX))
        .push_const(Value::U128(1))
        .push_var(op1)
        .push_var(op)
        .binary_op(Operation::Sub, SType::Number, SType::Number)?
        .binary_op(Operation::Add, SType::Number, SType::Number)?
        .binary_op(Operation::Sub, SType::Number, SType::Number)?
        .build(SType::Number)?;

    translator.mir.add_statement(Statement::IF {
        cnd,
        true_br: vec![Statement::CreateVar(result, true_br)],
        false_br: vec![Statement::CreateVar(result, Operation::Sub.expr(op, op1))],
    });
    Ok(())
}

fn plain_u128_ops(
    translator: &mut MirTranslator,
    cmd: Operation,
    op: Variable,
    op1: Variable,
    result: VarId,
) -> Result<(), Error> {
    let result = translator.map_var(result, SType::Number);

    let op = translator.cast_number(op)?;
    let op1 = translator.cast_number(op1)?;

    translator
        .mir
        .add_statement(Statement::CreateVar(result, cmd.expr(op, op1)));
    Ok(())
}

///
/// let revert_op1 = u128::max - op1;
/// if revert_op1 < op {
///     //overflow
///     op - revert_b - 1
/// } else {
///     op + op1
/// }
fn translate_add(
    translator: &mut MirTranslator,
    op: Variable,
    op1: Variable,
    result: VarId,
) -> Result<(), Error> {
    let result = translator.map_var(result, SType::Number);

    let cnd = StackOpsBuilder::default()
        .push_const(Value::U128(u128::MAX))
        .push_var(op1)
        .binary_op(Operation::Sub, SType::Number, SType::Number)?
        .push_var(op)
        .binary_op(Operation::Lt, SType::Number, SType::Bool)?
        .build(SType::Bool)?;

    let true_br = StackOpsBuilder::default()
        .push_var(op)
        .push_const(Value::U128(u128::MAX))
        .push_var(op1)
        .binary_op(Operation::Sub, SType::Number, SType::Number)?
        .binary_op(Operation::Sub, SType::Number, SType::Number)?
        .push_const(Value::U128(1))
        .binary_op(Operation::Sub, SType::Number, SType::Number)?
        .build(SType::Number)?;

    translator.mir.add_statement(Statement::IF {
        cnd,
        true_br: vec![Statement::CreateVar(result, true_br)],
        false_br: vec![Statement::CreateVar(result, Operation::Add.expr(op, op1))],
    });
    Ok(())
}
