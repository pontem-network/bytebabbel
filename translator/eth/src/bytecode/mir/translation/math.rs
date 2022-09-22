use crate::bytecode::hir::executor::math::{BinaryOp, TernaryOp, UnaryOp};
use crate::bytecode::hir::ir::var::{Expr, VarId};
use crate::bytecode::mir::ir::expression::{Expression, StackOpsBuilder, TypedExpression};
use crate::bytecode::mir::ir::types::SType;
use crate::MirTranslator;
use anyhow::{anyhow, Error};

impl<'a> MirTranslator<'a> {
    pub(super) fn translate_binary_op(
        &mut self,
        op: BinaryOp,
        arg: VarId,
        arg1: VarId,
    ) -> Result<TypedExpression, Error> {
        let arg = self.get_var(arg)?;
        let arg1 = self.get_var(arg1)?;

        let (arg, arg1) = if op == BinaryOp::Eq {
            if arg.ty() == SType::Bool && arg1.ty() == SType::Bool {
                (arg, arg1)
            } else {
                (self.cast(arg, SType::Num)?, self.cast(arg1, SType::Num)?)
            }
        } else {
            let arg = self.cast(arg, SType::Num)?;
            let arg1 = self.cast(arg1, SType::Num)?;
            (arg, arg1)
        };

        let ty = match op {
            BinaryOp::Eq | BinaryOp::Lt | BinaryOp::Gt | BinaryOp::SLt | BinaryOp::SGt => {
                SType::Bool
            }
            _ => SType::Num,
        };
        Ok(Expression::Binary(op, arg, arg1).ty(ty))
    }

    pub(super) fn translate_ternary_op(
        &mut self,
        op: TernaryOp,
        arg: VarId,
        arg1: VarId,
        arg2: VarId,
    ) -> Result<TypedExpression, Error> {
        let arg = self.get_var(arg)?;
        let arg1 = self.get_var(arg1)?;
        let arg2 = self.get_var(arg2)?;
        Ok(Expression::Ternary(op, arg, arg1, arg2).ty(SType::Num))
    }

    pub(super) fn translate_unary_op(
        &mut self,
        op: UnaryOp,
        arg: &Expr,
    ) -> Result<TypedExpression, Error> {
        let expr = self.translate_expr(arg)?;
        match expr.ty {
            SType::Num => Ok(self.unary_with_num(op, expr)),
            SType::Bool => self.unary_with_bool(op, expr),
            _ => Err(anyhow!(
                "Unary operation {:?} not supported for type {:?}",
                op,
                expr.ty
            )),
        }
    }

    fn unary_with_num(&mut self, op: UnaryOp, arg: TypedExpression) -> TypedExpression {
        match op {
            UnaryOp::IsZero => Expression::Unary(UnaryOp::IsZero, arg).ty(SType::Bool),
            UnaryOp::Not => Expression::Unary(UnaryOp::Not, arg).ty(SType::Num),
        }
    }

    fn unary_with_bool(
        &mut self,
        op: UnaryOp,
        args: TypedExpression,
    ) -> Result<TypedExpression, Error> {
        Ok(match op {
            UnaryOp::IsZero => StackOpsBuilder::default()
                .push_expr(args)?
                .push_const_bool(false)
                .eq()?
                .build(SType::Bool)?
                .ty(SType::Bool),
            UnaryOp::Not => StackOpsBuilder::default()
                .push_expr(args)?
                .not()?
                .build(SType::Bool)?
                .ty(SType::Bool),
        })
    }
}
