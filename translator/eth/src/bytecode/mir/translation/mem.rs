use crate::bytecode::hir::ir::{Expr, _Expr};
use crate::bytecode::loc::Loc;
use anyhow::{ensure, Error};

use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::SType;
use crate::MirTranslator;

impl<'a> MirTranslator<'a> {
    pub(super) fn translate_mem_store(&mut self, offset: Expr, val: Expr) -> Result<(), Error> {
        let offset = self.translate_expr(offset)?;
        let val = self.translate_expr(val)?;
        let val = self.cast_expr(val, SType::Num)?;

        ensure!(
            offset.ty == SType::Num,
            "Expected Number type for memory address"
        );
        self.mir.push(self.loc.wrap(Statement::MStore {
            memory: self.mem_var,
            offset,
            val,
        }));
        Ok(())
    }

    pub(super) fn translate_mem_store8(&mut self, offset: Expr, val: Expr) -> Result<(), Error> {
        let offset = self.translate_expr(offset)?;
        let val = self.translate_expr(val)?;
        let val = self.cast_expr(val, SType::Num)?;

        ensure!(
            offset.ty == SType::Num,
            "Expected Number type for memory address"
        );
        self.mir.push(self.loc.wrap(Statement::MStore8 {
            memory: self.mem_var,
            offset,
            val,
        }));
        Ok(())
    }
}
