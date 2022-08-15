use crate::bytecode::hir::executor::math::BinaryOp;
use crate::bytecode::hir::ir::var::VarId;
use crate::bytecode::mir::ir::math::Operation;
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::{LocalIndex, SType};
use crate::bytecode::mir::translation::Variable;
use crate::MirTranslator;
use anyhow::Error;
use std::rc::Rc;

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
                let result = self.map_local_var(result, SType::Bool);
                translate_eq(self, op, op1, result)?;
            }
            BinaryOp::Lt => {
                let op = self.cast_number(op)?;
                let op1 = self.cast_number(op1)?;
                let result = self.map_local_var(result, SType::Bool);
                let action = Statement::Operation(Operation::Lt, op, op1);
                self.mir
                    .add_statement(Statement::CreateVar(result, Box::new(action)));
            }
            BinaryOp::Gt => {
                todo!()
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
                todo!()
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
                let result = self.map_local_var(result, SType::Bool);
                translate_sub(self, op, op1, result)?;
                todo!()
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
    op: Rc<Variable>,
    op1: Rc<Variable>,
    result: LocalIndex,
) -> Result<(), Error> {
    ///todo optimize qe cases with consts
    let (op, op1) = if op.s_type() == op1.s_type() {
        (op, op1)
    } else {
        let op = translator.cast_number(op)?;
        let op1 = translator.cast_number(op1)?;
        (op, op1)
    };
    let action = Statement::Operation(Operation::Eq, op, op1);
    translator
        .mir
        .add_statement(Statement::CreateVar(result, Box::new(action)));
    Ok(())
}

///if b > a { // overflow u128::MAX - (b - a) + 1 } else { a - b }
fn translate_sub(
    translator: &mut MirTranslator,
    op: Rc<Variable>,
    op1: Rc<Variable>,
    result: LocalIndex,
) -> Result<(), Error> {
    let op = translator.cast_number(op)?;
    let op1 = translator.cast_number(op1)?;

    let cnd = translator.variables.borrow_local(SType::Bool);
    translator
        .mir
        .add_statement(Statement::CreateVar(cnd, Box::new(Statement::Operation(Operation::Gt, op1, op))));

    Statement::IF {
        cnd: translator.use_var(cnd)?,
        true_br: vec![],
        false_br: vec![]
    }


    let action = Statement::Operation(Operation::Eq, op, op1);
    translator
        .mir
        .add_statement(Statement::CreateVar(result, Box::new(action)));
    Ok(())
}
