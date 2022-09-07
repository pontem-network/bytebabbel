use crate::bytecode::hir::ir::var::{VarId, Vars};
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::SType;
use crate::MirTranslator;
use anyhow::{ensure, Error};

impl<'a> MirTranslator<'a> {
    pub(super) fn translate_s_store(
        &mut self,
        addr: VarId,
        var_id: VarId,
        _vars: &mut Vars,
    ) -> Result<(), Error> {
        let var = self.get_var(var_id)?;
        let addr = self.get_var(addr)?;
        let var = self.cast_number(var)?;

        ensure!(
            addr.s_type() == SType::Number,
            "Expected Number type for storage address"
        );

        self.mir.add_statement(Statement::SStore {
            storage: self.store_var,
            offset: addr,
            val: var,
        });
        Ok(())
    }
}
