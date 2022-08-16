use crate::bytecode::hir::ir::instruction::Instruction;
use crate::bytecode::hir::ir::var::{Var, VarId, Vars};
use crate::bytecode::mir::ir::expression::Expression;
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::{LocalIndex, SType, Value};
use crate::bytecode::mir::ir::Mir;
use crate::bytecode::mir::translation::mem::Memory;
use crate::bytecode::mir::translation::variables::{Variable, Variables};
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
    pub(super) mapping: HashMap<VarId, Variable>,
    pub(super) mir: Mir,
}

impl MirTranslator {
    pub fn new(fun: Function) -> MirTranslator {
        let variables = Variables::new(fun.input.len() as LocalIndex);
        MirTranslator {
            fun,
            variables,
            mem: Default::default(),
            mapping: Default::default(),
            mir: Default::default(),
        }
    }

    pub fn translate(mut self, hir: Hir) -> Result<Mir, Error> {
        let (mut vars, instructions) = hir.into_inner();
        self.translate_instructions(&instructions, &mut vars)?;
        self.mir.set_locals(self.variables.locals());
        Ok(self.mir)
    }

    pub(super) fn map_var(&mut self, var_id: VarId, tp: SType) -> Variable {
        let var = self.variables.borrow(tp);
        self.mapping.insert(var_id, var);
        var
    }

    fn translate_instructions(
        &mut self,
        instructions: &[Instruction],
        vars: &mut Vars,
    ) -> Result<(), Error> {
        let _scope = self.variables.create_scope();
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
                        .map(|id| self.get_var(*id))
                        .collect::<Result<Vec<_>, _>>()?;
                    self.mir.add_statement(Statement::Result(vars));
                }
                Instruction::MapVar { id, val } => {
                    let val = self.get_var(*val)?;
                    let id = self.get_var(*id)?;
                    self.mir.add_statement(Statement::CreateVar(id, val.expr()));
                }
                Instruction::Continue {
                    loop_id: id,
                    context: inst,
                } => {
                    self.translate_instructions(inst, vars)?;
                    self.mir.add_statement(Statement::Continue(*id));
                }
            }
        }
        Ok(())
    }

    fn translate_set_var(&mut self, id: VarId, vars: &mut Vars) -> Result<(), Error> {
        let var = vars.take(id)?;
        match var {
            Var::Val(val) => {
                let var = self.variables.borrow(SType::U128);
                self.mapping.insert(id, var);

                self.mir.add_statement(Statement::CreateVar(
                    var,
                    Expression::Const(Value::from(val)),
                ));
            }
            Var::Param(param_id) => {
                let param = self
                    .fun
                    .input
                    .get(param_id as usize)
                    .ok_or_else(|| anyhow!("parameter index out of bounds"))?;
                let tp = SType::from(param);
                let var = self.variables.borrow(tp);
                self.mapping.insert(id, var);
                self.mir.add_statement(Statement::CreateVar(
                    var,
                    Expression::Param(param_id as LocalIndex, tp),
                ));
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

    fn get_var(&mut self, id: VarId) -> Result<Variable, Error> {
        let var = self
            .mapping
            .get(&id)
            .ok_or_else(|| anyhow!("variable {:?} not found", id))?;
        Ok(*var)
    }
}
