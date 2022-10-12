use crate::bytecode::hir::ir::{Expr};

use anyhow::{ensure, Error};

use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::SType;
use crate::MirTranslator;

impl<'a> MirTranslator<'a> {
    pub(super) fn translate_s_store(&mut self, key: Expr, val: Expr) -> Result<(), Error> {
        let key = self.translate_expr(key)?;
        let val = self.translate_expr(val)?;
        let val = self.cast_expr(val, SType::Num)?;

        ensure!(
            key.ty == SType::Num,
            "Expected Number type for storage address"
        );

        self.mir.push(self.loc.wrap(Statement::SStore {
            storage: self.store_var,
            key,
            val,
        }));
        Ok(())
    }
}
