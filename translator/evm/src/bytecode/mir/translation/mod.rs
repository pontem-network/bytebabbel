use crate::bytecode::hir::ir::instruction::Instruction;
use crate::bytecode::hir::ir::var::{Var, VarId, Vars};
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::{LocalIndex, SType, Value};
use crate::bytecode::mir::ir::Mir;
use crate::bytecode::mir::translation::mem::Memory;
use crate::bytecode::mir::translation::variables::Variables;
use crate::{Function, Hir};
use anyhow::{anyhow, Error};
use std::collections::HashMap;

pub mod binary;
pub mod brunch;
pub mod cast;
pub mod consts;
pub mod mem;
pub mod unary;
pub mod variables;

pub struct MirTranslator {
    pub(super) fun: Function,
    pub(super) variables: Variables,
    pub(super) mem: Memory,
    pub(super) data_store: HashMap<VarId, Variable>,
    pub(super) mir: Mir,
}

impl MirTranslator {
    pub fn new(fun: Function) -> MirTranslator {
        let variables = Variables::new(fun.input.len() as LocalIndex);
        MirTranslator {
            fun,
            variables,
            mem: Default::default(),
            data_store: Default::default(),
            mir: Default::default(),
        }
    }

    pub fn translate_hir(mut self, hir: Hir) -> Result<Mir, Error> {
        let (mut vars, instructions) = hir.into_inner();
        self.translate_instructions(&instructions, &mut vars)?;
        Ok(self.mir)
    }

    pub(super) fn map_local_var(&mut self, var_id: VarId, tp: SType) -> LocalIndex {
        let result_var = self.variables.borrow_local(tp);
        self.data_store
            .insert(var_id, Variable::LocalBorrow(result_var, tp));
        result_var
    }

    fn translate_instructions(
        &mut self,
        instructions: &[Instruction],
        vars: &mut Vars,
    ) -> Result<(), Error> {
        for instruction in instructions {
            match instruction {
                Instruction::SetVar(id) => {
                    self.translate_set_var(*id, vars)?;
                }
                Instruction::MemStore(addr, var_id) => {
                    self.translate_mem_store(*addr, *var_id, vars)?;
                }
                Instruction::MemLoad(addr, var_id) => {
                    self.translate_mem_load(*addr, *var_id, vars)?;
                }
                Instruction::If {
                    condition,
                    true_branch,
                    false_branch,
                } => {
                    self.translate_if(*condition, true_branch, false_branch, vars)?;
                }
                Instruction::Loop {
                    id,
                    condition_block,
                    condition,
                    is_true_br_loop,
                    loop_br,
                } => {
                    self.translate_loop(
                        *id,
                        condition_block,
                        *condition,
                        *is_true_br_loop,
                        loop_br,
                        vars,
                    )?;
                }

                Instruction::Stop => {
                    self.mir.add_statement(Statement::Abort(u8::MAX));
                }
                Instruction::Abort(code) => {
                    self.mir.add_statement(Statement::Abort(*code));
                }
                Instruction::Result(vars) => {
                    let vars = vars
                        .iter()
                        .map(|id| self.use_var(*id))
                        .collect::<Result<Vec<_>, _>>()?;
                    self.mir.add_statement(Statement::Result(vars));
                }
                Instruction::MapVar { id: _, val: _ } => {
                    todo!()
                }
                Instruction::Continue {
                    loop_id: _,
                    context: _,
                } => {
                    todo!()
                }
            }
        }
        Ok(())
    }

    fn translate_set_var(&mut self, id: VarId, vars: &mut Vars) -> Result<(), Error> {
        let var = vars.take(id)?;
        match var {
            Var::Val(val) => {
                self.set_const(id, val);
            }
            Var::Param(param_id) => {
                let param = self
                    .fun
                    .input
                    .get(param_id as usize)
                    .ok_or_else(|| anyhow!("parameter index out of bounds"))?;
                self.data_store.insert(
                    id,
                    Variable::ParamAlias(param_id as LocalIndex, SType::from(param)),
                );
            }
            Var::UnaryOp(cmd, op) => {
                self.translate_unary_op(cmd, op, id)?;
            }
            Var::BinaryOp(cmd, op1, op2) => {
                self.translate_binary_op(cmd, op1, op2, id)?;
            }
            Var::TernaryOp(_, _, _, _) => {
                todo!()
            }
        }
        Ok(())
    }

    fn set_const<C: Into<Value>>(&mut self, id: VarId, val: C) {
        let value = val.into();
        let stype = value.s_type();
        self.data_store.insert(id, Variable::Const(value, stype));
    }

    fn use_var(&mut self, id: VarId) -> Result<Variable, Error> {
        let var = self
            .data_store
            .get(&id)
            .ok_or_else(|| anyhow!("variable {:?} not found", id))?
            .clone();

        if let Variable::LocalBorrow(index, _) = &var {
            self.data_store.remove(&id);
            self.variables.release_local(*index);
        }
        Ok(var)
    }
}

#[derive(Debug, Clone)]
pub enum Variable {
    Const(Value, SType),
    ParamAlias(LocalIndex, SType),
    LocalBorrow(LocalIndex, SType),
}

impl Variable {
    pub fn s_type(&self) -> SType {
        match self {
            Variable::Const(_, stype) => *stype,
            Variable::ParamAlias(_, stype) => *stype,
            Variable::LocalBorrow(_, stype) => *stype,
        }
    }
}
