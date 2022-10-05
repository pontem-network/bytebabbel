use anyhow::{anyhow, Error};

use crate::bytecode::hir::executor::math::{BinaryOp, TernaryOp, UnaryOp};
use crate::bytecode::hir::ir::Expr;
use crate::bytecode::mir::ir::expression::{Expression, StackOpsBuilder, TypedExpr};
use crate::bytecode::mir::ir::types::SType;
use crate::MirTranslator;

impl<'a> MirTranslator<'a> {
    pub(super) fn translate_binary_op(
        &mut self,
        op: BinaryOp,
        arg: &Expr,
        arg1: &Expr,
    ) -> Result<TypedExpr, Error> {
        let arg = self.translate_expr(arg)?;
        let arg1 = self.translate_expr(arg1)?;

        let (arg, arg1) = if op == BinaryOp::Eq {
            if arg.ty == SType::Bool && arg1.ty == SType::Bool {
                (arg, arg1)
            } else {
                (
                    self.cast_expr(arg, SType::Num)?,
                    self.cast_expr(arg1, SType::Num)?,
                )
            }
        } else {
            let arg = self.cast_expr(arg, SType::Num)?;
            let arg1 = self.cast_expr(arg1, SType::Num)?;
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
        arg: &Expr,
        arg1: &Expr,
        arg2: &Expr,
    ) -> Result<TypedExpr, Error> {
        let arg = self.translate_expr(arg)?;
        let arg1 = self.translate_expr(arg1)?;
        let arg2 = self.translate_expr(arg2)?;
        Ok(Expression::Ternary(
            op,
            self.cast_expr(arg, SType::Num)?,
            self.cast_expr(arg1, SType::Num)?,
            self.cast_expr(arg2, SType::Num)?,
        )
        .ty(SType::Num))
    }

    pub(super) fn translate_unary_op(
        &mut self,
        op: UnaryOp,
        arg: &Expr,
    ) -> Result<TypedExpr, Error> {
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

    fn unary_with_num(&mut self, op: UnaryOp, arg: TypedExpr) -> TypedExpr {
        match op {
            UnaryOp::IsZero => Expression::Unary(UnaryOp::IsZero, arg).ty(SType::Bool),
            UnaryOp::Not => Expression::Unary(UnaryOp::Not, arg).ty(SType::Num),
        }
    }

    fn unary_with_bool(&mut self, op: UnaryOp, args: TypedExpr) -> Result<TypedExpr, Error> {
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
