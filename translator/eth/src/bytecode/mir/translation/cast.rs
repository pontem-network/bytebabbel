use anyhow::Error;

use crate::bytecode::mir::ir::expression::{Cast, Expression, TypedExpr};
use crate::bytecode::mir::ir::types::SType;
use crate::bytecode::mir::translation::Variable;
use crate::MirTranslator;

impl<'a> MirTranslator<'a> {
    pub fn cast(&mut self, from: Variable, to: SType) -> Result<Variable, Error> {
        if from.ty() == to {
            return Ok(from);
        }

        let res = self.cast_expr(from.expr(), to)?;
        let var = self.variables.borrow_global(to);
        self.mir.push(var.assign(res));
        Ok(var)
    }

    pub fn cast_expr(&mut self, from: TypedExpr, to: SType) -> Result<TypedExpr, Error> {
        if from.ty == to {
            return Ok(from);
        }
        let cast = Cast::make(from.ty, to)?;
        Ok(Expression::Cast(from, cast).ty(to))
    }
}
