use crate::bytecode::hir2::executor::math::{BinaryOp, TernaryOp, UnaryOp};
use crate::bytecode::hir2::ir::expression::Expr;
use crate::bytecode::mir::ir::expression::{Expression, StackOpsBuilder, TypedExpr};
use crate::bytecode::mir::ir::types::SType;
use crate::MirTranslator;
use anyhow::{bail, Error};

impl<'a> MirTranslator<'a> {
    pub(super) fn translate_binary_op(
        &mut self,
        cmd: BinaryOp,
        op: &Expr,
        op1: &Expr,
    ) -> Result<TypedExpr, Error> {
        let op = self.expr(op)?;
        let op1 = self.expr(op1)?;

        let (op, op1) = if cmd == BinaryOp::Eq {
            if op.ty == SType::Bool && op1.ty == SType::Bool {
                (op, op1)
            } else {
                (self.cast(op, SType::Num)?, self.cast(op1, SType::Num)?)
            }
        } else {
            let op = self.cast(op, SType::Num)?;
            let op1 = self.cast(op1, SType::Num)?;
            (op, op1)
        };

        Ok(cmd.expr(op, op1))
    }

    pub(super) fn translate_unary_op(
        &mut self,
        cmd: UnaryOp,
        op: &Expr,
    ) -> Result<TypedExpr, Error> {
        let op = self.expr(op)?;
        Ok(match op.ty {
            SType::Num => match cmd {
                UnaryOp::IsZero => Expression::UnOp(cmd, op).ty(SType::Bool),
                UnaryOp::Not => Expression::UnOp(cmd, op).ty(SType::Num),
            },
            SType::Bool => match cmd {
                UnaryOp::IsZero => StackOpsBuilder::default()
                    .push_bool(op)?
                    .push_const_bool(false)
                    .eq()?
                    .build(SType::Bool)?,
                UnaryOp::Not => StackOpsBuilder::default()
                    .push_bool(op)?
                    .not()?
                    .build(SType::Bool)?,
            },
            _ => bail!(
                "Unary operation {:?} not supported for type {:?}",
                cmd,
                op.ty
            ),
        })
    }

    pub(super) fn translate_ternary_op(
        &mut self,
        cmd: TernaryOp,
        op: &Expr,
        op1: &Expr,
        op2: &Expr,
    ) -> Result<TypedExpr, Error> {
        let op = self.expr(op)?;
        let op1 = self.expr(op1)?;
        let op2 = self.expr(op2)?;
        let op = self.cast(op, SType::Num)?;
        let op1 = self.cast(op1, SType::Num)?;
        let op2 = self.cast(op2, SType::Num)?;
        Ok(cmd.expr(op, op1, op2))
    }
}

impl BinaryOp {
    pub fn expr(self, op: TypedExpr, op1: TypedExpr) -> TypedExpr {
        let ty = match self {
            BinaryOp::Eq | BinaryOp::Lt | BinaryOp::Gt | BinaryOp::SLt | BinaryOp::SGt => {
                SType::Bool
            }
            _ => SType::Num,
        };
        Expression::BinOp(self, op, op1).ty(ty)
    }
}

impl TernaryOp {
    pub fn expr(self, op: TypedExpr, op1: TypedExpr, op2: TypedExpr) -> TypedExpr {
        Expression::TernOp(self, op, op1, op2).ty(SType::Num)
    }
}
