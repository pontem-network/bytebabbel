use crate::bytecode::hir::ir::VarId;
use crate::bytecode::hir::vars::Vars;
use anyhow::{ensure, Error};

use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::SType;
use crate::MirTranslator;

impl<'a> MirTranslator<'a> {
    pub(super) fn translate_s_store(
        &mut self,
        addr: VarId,
        var_id: VarId,
        _vars: &mut Vars,
    ) -> Result<(), Error> {
        let var = self.get_var(var_id)?;
        let addr = self.get_var(addr)?;
        let var = self.cast(var, SType::Num)?;

        ensure!(
            addr.ty() == SType::Num,
            "Expected Number type for storage address"
        );

        self.mir.push(Statement::SStore {
            storage: self.store_var,
            key: addr,
            val: var,
        });
        Ok(())
    }
}
