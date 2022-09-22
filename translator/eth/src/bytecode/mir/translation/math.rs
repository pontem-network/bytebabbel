use crate::bytecode::hir::executor::math::{BinaryOp, TernaryOp, UnaryOp};
use crate::bytecode::hir::ir::var::VarId;
use crate::bytecode::mir::ir::expression::{Expression, StackOpsBuilder};
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::SType;
use crate::bytecode::mir::translation::variables::Variable;
use crate::MirTranslator;
use anyhow::{anyhow, Error};

impl<'a> MirTranslator<'a> {
    pub(super) fn translate_binary_op(
        &mut self,
        op: BinaryOp,
        arg: VarId,
        arg1: VarId,
        result: VarId,
    ) -> Result<(), Error> {
        let arg = self.get_var(arg)?;
        let arg1 = self.get_var(arg1)?;

        let (arg, arg1) = if op == BinaryOp::Eq {
            if arg.s_type() == SType::Bool && arg1.s_type() == SType::Bool {
                (arg, arg1)
            } else {
                (self.cast(arg, SType::Num)?, self.cast(arg1, SType::Num)?)
            }
        } else {
            let arg = self.cast(arg, SType::Num)?;
            let arg1 = self.cast(arg1, SType::Num)?;
            (arg, arg1)
        };

        let result = match op {
            BinaryOp::Eq | BinaryOp::Lt | BinaryOp::Gt | BinaryOp::SLt | BinaryOp::SGt => {
                self.map_var(result, SType::Bool)
            }
            _ => self.map_var(result, SType::Num),
        };
        self.mir
            .push(Statement::Assign(result, Expression::Binary(op, arg, arg1)));
        Ok(())
    }

    pub(super) fn translate_ternary_op(
        &mut self,
        op: TernaryOp,
        arg: VarId,
        arg1: VarId,
        arg2: VarId,
        result: VarId,
    ) -> Result<(), Error> {
        let arg = self.get_var(arg)?;
        let arg1 = self.get_var(arg1)?;
        let arg2 = self.get_var(arg2)?;
        let result = self.map_var(result, SType::Num);
        self.mir
            .push(result.assign(Expression::Ternary(op, arg, arg1, arg2)));
        Ok(())
    }

    pub(super) fn translate_unary_op(
        &mut self,
        op: UnaryOp,
        arg: VarId,
        result: VarId,
    ) -> Result<(), Error> {
        let var = self.get_var(arg)?;
        match var.s_type() {
            SType::Num => self.unary_with_num(op, var, result),
            SType::Bool => self.unary_with_bool(op, var, result),
            _ => Err(anyhow!(
                "Unary operation {:?} not supported for type {:?}",
                op,
                var.s_type()
            )),
        }
    }

    fn unary_with_num(&mut self, op: UnaryOp, arg: Variable, result: VarId) -> Result<(), Error> {
        match op {
            UnaryOp::IsZero => {
                let result = self.map_var(result, SType::Bool);
                self.mir
                    .push(result.assign(Expression::Unary(UnaryOp::IsZero, arg)));
            }
            UnaryOp::Not => {
                let result = self.map_var(result, SType::Num);
                self.mir
                    .push(result.assign(Expression::Unary(UnaryOp::Not, arg)));
            }
        }
        Ok(())
    }

    fn unary_with_bool(&mut self, op: UnaryOp, args: Variable, result: VarId) -> Result<(), Error> {
        let result = self.map_var(result, SType::Bool);
        match op {
            UnaryOp::IsZero => {
                let ops = StackOpsBuilder::default()
                    .push_bool(args)?
                    .push_const_bool(false)
                    .eq()?
                    .build(SType::Bool)?;
                self.mir.push(Statement::Assign(result, ops));
            }
            UnaryOp::Not => {
                let ops = StackOpsBuilder::default()
                    .push_bool(args)?
                    .not()?
                    .build(SType::Bool)?;
                self.mir.push(Statement::Assign(result, ops));
            }
        }
        Ok(())
    }
}
