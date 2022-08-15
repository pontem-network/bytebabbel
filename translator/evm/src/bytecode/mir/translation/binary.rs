use crate::bytecode::hir::executor::math::BinaryOp;
use crate::bytecode::hir::ir::var::VarId;
use crate::bytecode::mir::ir::expression::StackOpsBuilder;
use crate::bytecode::mir::ir::math::Operation;
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::{LocalIndex, SType, Value};
use crate::bytecode::mir::translation::Variable;
use crate::MirTranslator;
use anyhow::Error;

impl MirTranslator {
    pub(super) fn translate_binary_op(
        &mut self,
        cmd: BinaryOp,
        op: VarId,
        op1: VarId,
        result: VarId,
    ) -> Result<(), Error> {
        let op = self.use_var(op)?;
        let op1 = self.use_var(op1)?;

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
                todo!()
            }
            BinaryOp::Shl => {
                todo!()
            }
            BinaryOp::Sar => {
                todo!()
            }
            BinaryOp::Add => {
                translate_add(self, op, op1, result)?;
            }
            BinaryOp::And => {
                todo!()
            }
            BinaryOp::Or => {
                todo!()
            }
            BinaryOp::Xor => {
                todo!()
            }
            BinaryOp::Mul => {
                todo!()
            }
            BinaryOp::Sub => {
                translate_sub(self, op, op1, result)?;
            }
            BinaryOp::Div => {
                todo!()
            }
            BinaryOp::SDiv => {
                todo!()
            }
            BinaryOp::SLt => {
                todo!()
            }
            BinaryOp::SGt => {
                todo!()
            }
            BinaryOp::Byte => {
                todo!()
            }
            BinaryOp::Mod => {
                todo!()
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

fn translate_eq(
    translator: &mut MirTranslator,
    op: Variable,
    op1: Variable,
    result: LocalIndex,
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
    let result = translator.map_var(result, SType::Bool);

    let op = translator.cast_number(op)?;
    let op1 = translator.cast_number(op1)?;

    let mut cnd = StackOpsBuilder::default();
    cnd.push_var(op1.clone());
    cnd.push_var(op.clone());
    cnd.binary_op(Operation::Gt, SType::U128, SType::Bool)?;
    let cnd = cnd.build(SType::Bool)?;

    let mut true_br = StackOpsBuilder::default();
    true_br.push_const(Value::U128(u128::MAX));
    true_br.push_const(Value::U128(1));
    true_br.push_var(op1.clone());
    true_br.push_var(op.clone());
    true_br.binary_op(Operation::Sub, SType::U128, SType::U128)?;
    true_br.binary_op(Operation::Add, SType::U128, SType::U128)?;
    true_br.binary_op(Operation::Sub, SType::U128, SType::U128)?;

    translator.mir.add_statement(Statement::IF {
        cnd,
        true_br: vec![Statement::CreateVar(result, true_br.build(SType::U128)?)],
        false_br: vec![Statement::CreateVar(result, Operation::Sub.expr(op, op1))],
    });
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
    let result = translator.map_var(result, SType::Bool);

    let mut cnd = StackOpsBuilder::default();
    cnd.push_const(Value::U128(u128::MAX));
    cnd.push_var(op1.clone());
    cnd.binary_op(Operation::Sub, SType::U128, SType::U128)?;
    cnd.push_var(op.clone());
    cnd.binary_op(Operation::Lt, SType::U128, SType::Bool)?;

    let mut true_br = StackOpsBuilder::default();
    true_br.push_var(op.clone());
    true_br.push_const(Value::U128(u128::MAX));
    true_br.push_var(op1.clone());
    true_br.binary_op(Operation::Sub, SType::U128, SType::U128)?;
    true_br.binary_op(Operation::Sub, SType::U128, SType::U128)?;
    true_br.push_const(Value::U128(1));
    true_br.binary_op(Operation::Sub, SType::U128, SType::U128)?;

    translator.mir.add_statement(Statement::IF {
        cnd: cnd.build(SType::Bool)?,
        true_br: vec![Statement::CreateVar(result, true_br.build(SType::U128)?)],
        false_br: vec![Statement::CreateVar(result, Operation::Add.expr(op, op1))],
    });
    Ok(())
}
