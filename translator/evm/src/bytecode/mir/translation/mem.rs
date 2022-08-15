use crate::bytecode::hir::ir::var::{VarId, Vars};
use crate::bytecode::mir::ir::types::LocalIndex;
use crate::{MirTranslator, U256};
use anyhow::Error;
use std::collections::HashMap;

#[derive(Default, Debug, Clone)]
pub struct Memory {
    mapping: HashMap<U256, LocalIndex>,
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
        let var = self.use_var(var_id)?;

        let local = self
            .mem
            .mapping
            .entry(addr)
            .or_insert_with(|| self.variables.borrow(var.s_type()));
        self.variables.check_type(var.s_type(), *local)?;
        todo!();
        // match &var {
        //     Variable::Const(val, _) => {
        //         self.mir.add_statement(Statement::CreateVar(
        //             *local,
        //             Box::new(Statement::Const(val.clone())),
        //         ));
        //     }
        //     Variable::ParamAlias(val, _) => {
        //         self.mir.add_statement(Statement::CreateVar(
        //             *local,
        //             Box::new(Statement::Param(*val)),
        //         ));
        //     }
        //     Variable::LocalBorrow(val, _) => {
        //         self.mir
        //             .add_statement(Statement::CreateVar(*local, Box::new(Statement::Var(*val))));
        //     }
        // }
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
