use crate::bytecode::hir::ir::instruction::Instruction;
use crate::bytecode::hir::ir::var::{Var, VarId, Vars};
use crate::bytecode::mir::ir::expression::{Expression, StackOpsBuilder};
use crate::bytecode::mir::ir::math::Operation;
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::{LocalIndex, SType, Value};
use crate::bytecode::mir::ir::Mir;
use crate::bytecode::mir::translation::variables::{Variable, Variables};
use crate::bytecode::types::EthType;
use crate::{Function, Hir};
use anyhow::{anyhow, ensure, Error};
use std::collections::HashMap;

pub mod binary;
pub mod brunch;
pub mod cast;
pub mod consts;
pub mod mem;
pub mod storage;
pub mod ternary;
pub mod unary;
pub mod variables;

pub struct MirTranslator<'a> {
    pub(super) fun: &'a Function,
    pub(super) variables: Variables,
    pub(super) mapping: HashMap<VarId, Variable>,
    pub(super) mir: Mir,
    pub(super) mem_var: Variable,
    pub(super) store_var: Variable,
}

impl<'a> MirTranslator<'a> {
    pub fn new(fun: &'a Function, is_constructor: bool) -> MirTranslator<'a> {
        let mut variables = Variables::new(fun.input.iter().map(|t| SType::from(t)).collect());
        let mut mir = Mir::default();

        if is_constructor {
            mir.add_statement(Statement::InitStorage(variables.borrow_param(0)));
        }

        let store_var = variables.borrow_global(SType::Storage);
        mir.add_statement(Statement::CreateVar(store_var, Expression::GetStore));

        let mem_var = variables.borrow_global(SType::Memory);
        mir.add_statement(Statement::CreateVar(mem_var, Expression::GetMem));

        MirTranslator {
            fun,
            variables,
            mapping: Default::default(),
            mir,
            mem_var,
            store_var,
        }
    }

    pub fn translate(mut self, hir: Hir) -> Result<Mir, Error> {
        let (mut vars, instructions, _) = hir.into_inner();
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
                Instruction::MemStore { addr, var } => {
                    self.translate_mem_store(*addr, *var, vars)?;
                }
                Instruction::MemStore8 { addr, var } => {
                    self.translate_mem_store8(*addr, *var, vars)?;
                }
                Instruction::SStore { addr, var } => {
                    self.translate_s_store(*addr, *var, vars)?;
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
                    if !self.fun.output.is_empty() {
                        self.mir.add_statement(Statement::Abort(u8::MAX));
                    } else {
                        self.translate_ret_unit()?;
                    }
                }
                Instruction::Abort(code) => {
                    self.mir.add_statement(Statement::Abort(*code));
                }
                Instruction::Result { offset, len } => {
                    self.translate_ret(*offset, *len)?;
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
                let var = self.variables.borrow(SType::Number);
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
            Var::TernaryOp(cmd, op1, op2, op3) => {
                self.translate_ternary_op(cmd, op1, op2, op3, id)?;
            }
            Var::MLoad(addr) => {
                let result = self.variables.borrow(SType::Number);
                let addr = self.get_var(addr)?;
                ensure!(
                    addr.s_type() == SType::Number,
                    "address must be of type u128"
                );
                self.mapping.insert(id, result);
                self.mir.add_statement(Statement::CreateVar(
                    result,
                    Expression::MLoad {
                        memory: self.mem_var,
                        offset: addr,
                    },
                ));
            }
            Var::SLoad(addr) => {
                let result = self.variables.borrow(SType::Number);
                self.mapping.insert(id, result);
                let addr = self.get_var(addr)?;
                ensure!(
                    addr.s_type() == SType::Number,
                    "address must be of type u128"
                );

                self.mir.add_statement(Statement::CreateVar(
                    result,
                    Expression::SLoad {
                        storage: self.store_var,
                        offset: addr,
                    },
                ));
            }
            Var::MSize => {
                let result = self.variables.borrow(SType::Number);
                self.mir.add_statement(Statement::CreateVar(
                    result,
                    Expression::MSize {
                        memory: self.mem_var,
                    },
                ));
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

    fn translate_ret_unit(&mut self) -> Result<(), Error> {
        self.mir.add_statement(Statement::Result(vec![]));
        Ok(())
    }

    fn translate_ret(&mut self, offset: VarId, _: VarId) -> Result<(), Error> {
        let mut vars = Vec::with_capacity(self.fun.output.len());
        let offset = self.get_var(offset)?;
        for output in self.fun.output.clone() {
            let var = self.variables.borrow(SType::Number);
            self.mir.add_statement(Statement::CreateVar(
                var,
                Expression::MLoad {
                    memory: self.mem_var,
                    offset,
                },
            ));
            self.mir.add_statement(Statement::CreateVar(
                offset,
                StackOpsBuilder::default()
                    .push_var(offset)
                    .push_const(Value::U128(32))
                    .binary_op(Operation::Add, SType::Number, SType::Number)?
                    .build(SType::Number)?,
            ));

            if let EthType::Bool = output {
                vars.push(self.cast_bool(var)?);
            } else {
                vars.push(var);
            }
        }
        self.mir.add_statement(Statement::Result(vars));
        Ok(())
    }
}
