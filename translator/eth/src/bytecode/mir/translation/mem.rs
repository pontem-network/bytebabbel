use crate::bytecode::hir::ir::var::{VarId, Vars};
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::SType;
use crate::MirTranslator;
use anyhow::{ensure, Error};

impl<'a> MirTranslator<'a> {
    pub(super) fn translate_mem_store(
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
            "Expected Number type for memory address"
        );
        self.mir.add_statement(Statement::MStore {
            memory: self.mem_var,
            offset: addr,
            val: var,
        });
        Ok(())
    }

    pub(super) fn translate_mem_store8(
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
            "Expected Number type for memory address"
        );
        self.mir.add_statement(Statement::MStore8 {
            memory: self.mem_var,
            offset: addr,
            val: var,
        });
        Ok(())
    }
}