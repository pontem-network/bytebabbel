use crate::bytecode::hir::ir::var::{VarId, Vars};
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::LocalIndex;
use crate::bytecode::mir::translation::Variable;
use crate::{MirTranslator, U256};
use anyhow::Error;
use std::collections::HashMap;
use std::rc::Rc;

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
        vars: &mut Vars,
    ) -> Result<(), Error> {
        //todo dynamic memory
        let var = self.use_var(var_id)?;

        let local = self
            .mem
            .mapping
            .entry(addr)
            .or_insert_with(|| self.variables.borrow_local(var.s_type()));
        self.variables.check_type(var.s_type(), *local)?;

        match &var {
            Variable::Const(val, st) => {
                self.mir.add_statement(Statement::CreateVar(
                    *local,
                    Box::new(Statement::Const(val.clone())),
                ));
            }
            Variable::ParamAlias(local_index, st) => {
                todo!()
            }
            Variable::LocalBorrow(local_index, st) => {
                todo!()
            }
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

    fn store_mem_var(&mut self, id: VarId, var: Variable) {
        self.data_store.insert(id, var);
    }
}
