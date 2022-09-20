use crate::bytecode::mir::ir::expression::{Cast, Expression, TypedExpr};
use crate::bytecode::mir::ir::types::SType;
use crate::MirTranslator;
use anyhow::Error;

impl<'a> MirTranslator<'a> {
    pub fn cast(&mut self, from: TypedExpr, to: SType) -> Result<TypedExpr, Error> {
        if from.ty == to {
            return Ok(from);
        }
        let cast = Cast::make(from.ty, to)?;
        Ok(Expression::Cast(from, cast).ty(to))
    }
}
