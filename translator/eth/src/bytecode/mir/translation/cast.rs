use crate::bytecode::loc::Loc;
use anyhow::Error;

use crate::bytecode::mir::ir::expression::{Cast, Expression, TypedExpr};
use crate::bytecode::mir::ir::types::SType;
use crate::bytecode::mir::translation::Variable;
use crate::MirTranslator;

impl<'a> MirTranslator<'a> {
    pub fn cast_expr(&mut self, from: Loc<TypedExpr>, to: SType) -> Result<Loc<TypedExpr>, Error> {
        if from.ty == to {
            return Ok(from);
        }
        let cast = Cast::make(from.ty, to)?;
        Ok(from.wrap(Expression::Cast(from, cast).ty(to)))
    }
}
