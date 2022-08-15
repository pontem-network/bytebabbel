use crate::bytecode::hir::ir::var::{VarId, Vars};
use crate::bytecode::mir::ir::expression::Expression;
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::translation::variables::Variable;
use crate::{MirTranslator, U256};
use anyhow::{ensure, Error};
use std::collections::HashMap;

#[derive(Default, Debug, Clone)]
pub struct Memory {
    mapping: HashMap<U256, Variable>,
}

impl Memory {}

impl MirTranslator {
    pub(super) fn translate_mem_store(
        &mut self,
        addr: U256,
        var_id: VarId,
        _vars: &mut Vars,
    ) -> Result<(), Error> {
        //todo dynamic memory
        let var = self.get_var(var_id)?;

        let local = self
            .mem
            .mapping
            .entry(addr)
            .or_insert_with(|| self.variables.borrow_global(var.s_type()));
        ensure!(local.s_type() == var.s_type(), "type mismatch");

        self.mir
            .add_statement(Statement::CreateVar(*local, Expression::Var(var)));
        Ok(())
    }

    pub(super) fn translate_mem_load(
        &mut self,
        _: U256,
        _var_id: VarId,
        _vars: &mut Vars,
    ) -> Result<(), Error> {
        Ok(())
    }
}
