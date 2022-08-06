use crate::bytecode::hir::ir::var::{VarId, Vars};
use crate::bytecode::mir::translation::Variable;
use crate::{MirTranslator, U256};
use anyhow::Error;
use std::rc::Rc;

#[derive(Default, Debug, Clone)]
pub struct Memory {}

impl Memory {}

impl MirTranslator {
    pub(super) fn translate_mem_store(
        &mut self,
        _: U256,
        var_id: VarId,
        vars: &mut Vars,
    ) -> Result<(), Error> {
        //todo dynamic memory
        let var = self.use_var(var_id)?;

        match var.as_ref() {
            Variable::Const(val, st) => {}
            Variable::ParamAlias(local_index, st) => {}
            Variable::LocalBorrow(local_index, st) => {}
        }

        Ok(())
    }

    pub(super) fn translate_mem_load(
        &mut self,
        _: U256,
        var_id: VarId,
        vars: &mut Vars,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn store_mem_var(&mut self, id: VarId, var: Rc<Variable>) {
        self.data_store.insert(id, var);
    }
}
