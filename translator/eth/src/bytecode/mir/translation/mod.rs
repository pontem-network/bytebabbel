use crate::bytecode::hir::executor::math::BinaryOp;
use crate::bytecode::hir::ir::statement::Statement as HirStmt;
use crate::bytecode::hir::ir::var::{VarId, Vars};
use crate::bytecode::mir::ir::expression::Expression;
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::{LocalIndex, SType, Value};
use crate::bytecode::mir::ir::Mir;
use crate::bytecode::mir::translation::variables::{Variable, Variables};
use crate::{Flags, Function, Hir};
use anyhow::{anyhow, Error};
use primitive_types::U256;
use std::collections::HashMap;

pub mod brunch;
pub mod cast;
pub mod consts;
pub mod expr;
pub mod math;
pub mod mem;
pub mod storage;
pub mod variables;

pub struct MirTranslator<'a> {
    pub(super) fun: &'a Function,
    pub(super) variables: Variables,
    pub(super) mapping: HashMap<VarId, Variable>,
    pub(super) mir: Mir,
    pub(super) mem_var: Variable,
    pub(super) store_var: Variable,
    pub(super) signer_index: LocalIndex,
    pub(super) args_index: LocalIndex,
    pub(super) flags: Flags,
}

impl<'a> MirTranslator<'a> {
    pub fn new(fun: &'a Function, flags: Flags) -> MirTranslator<'a> {
        let signer = (0, SType::Signer);
        let args = (1, SType::Bytes);

        let mut variables = if flags.native_input {
            let mut args = vec![SType::Signer];
            args.extend(
                fun.native_input
                    .iter()
                    .map(|t| SType::from_eth_type(t, flags.u128_io)),
            );
            Variables::new(args)
        } else {
            Variables::new(vec![signer.1, args.1])
        };

        let mut mir = Mir::default();

        let store_var = variables.borrow_global(SType::Storage);
        mir.push(store_var.assign(Expression::GetStore.ty(SType::Storage)));

        let mem_var = variables.borrow_global(SType::Memory);
        mir.push(mem_var.assign(Expression::GetMem.ty(SType::Memory)));

        MirTranslator {
            fun,
            variables,
            mapping: Default::default(),
            mir,
            mem_var,
            store_var,
            signer_index: signer.0,
            args_index: args.0,
            flags,
        }
    }

    pub fn translate(mut self, hir: Hir) -> Result<Mir, Error> {
        let (mut vars, instructions, _) = hir.into_inner();
        self.translate_instructions(&instructions, &mut vars)?;
        self.mir.set_locals(self.variables.locals());
        Ok(self.mir)
    }

    fn translate_instructions(
        &mut self,
        instructions: &[HirStmt],
        vars: &mut Vars,
    ) -> Result<(), Error> {
        let _scope = self.variables.create_scope();
        for instruction in instructions {
            match instruction {
                HirStmt::SetVar(id) => {
                    self.translate_set_var(*id, vars)?;
                }
                HirStmt::MemStore { addr, var } => {
                    self.translate_mem_store(*addr, *var, vars)?;
                }
                HirStmt::MemStore8 { addr, var } => {
                    self.translate_mem_store8(*addr, *var, vars)?;
                }
                HirStmt::SStore { addr, var } => {
                    self.translate_s_store(*addr, *var, vars)?;
                }
                HirStmt::If {
                    condition,
                    true_branch,
                    false_branch,
                } => {
                    self.translate_if(*condition, true_branch, false_branch, vars)?;
                }
                HirStmt::Loop {
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
                HirStmt::Stop => {
                    if !self.fun.native_output.is_empty() {
                        self.mir.push(Statement::Abort(u8::MAX));
                    } else {
                        self.translate_ret_unit()?;
                    }
                }
                HirStmt::Abort(code) => {
                    self.mir.push(Statement::Abort(*code));
                }
                HirStmt::Result { offset, len } => {
                    self.translate_ret(*offset, *len)?;
                }
                HirStmt::MapVar { id, val } => {
                    let val = self.get_var(*val)?;
                    let id = self.get_var(*id)?;
                    self.mir.push(Statement::Assign(id, val.expr()));
                }
                HirStmt::Continue {
                    loop_id: id,
                    context: inst,
                } => {
                    self.translate_instructions(inst, vars)?;
                    self.mir.push(Statement::Continue(*id));
                }
                HirStmt::Log {
                    offset,
                    len,
                    topics,
                } => {
                    let topics = topics
                        .iter()
                        .map(|t| self.get_var(*t))
                        .collect::<Result<Vec<_>, _>>()?;

                    let offset = self.get_var(*offset)?;
                    let len = self.get_var(*len)?;
                    self.mir.push(Statement::Log {
                        storage: self.store_var,
                        memory: self.mem_var,
                        offset,
                        len,
                        topics,
                    });
                }
            }
        }
        Ok(())
    }

    fn translate_set_var(&mut self, id: VarId, vars: &mut Vars) -> Result<(), Error> {
        let expr = self.translate_expr(&vars.take(id)?)?;
        let var = self.variables.borrow(expr.ty);
        self.mapping.insert(id, var);
        self.mir.push(var.assign(expr));
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
        if self.flags.hidden_output || self.flags.native_output {
            self.mir.push(Statement::Result(vec![]));
            return Ok(());
        }

        let unit = self.variables.borrow(SType::Bytes);
        let len = self.variables.borrow(SType::Num);
        self.mir.push(Statement::Assign(
            len,
            Expression::Const(Value::from(U256::zero())).ty(SType::Num),
        ));
        self.mir.push(Statement::Assign(
            unit,
            Expression::MSlice {
                memory: self.mem_var,
                offset: len,
                len,
            }
            .ty(SType::Bytes),
        ));
        self.mir.push(Statement::Result(vec![unit]));
        Ok(())
    }

    fn translate_ret(&mut self, offset: VarId, len: VarId) -> Result<(), Error> {
        if self.flags.hidden_output {
            self.mir.push(Statement::Result(vec![]));
            return Ok(());
        }

        if self.flags.native_output {
            let mut results = vec![];
            let offset = self.get_var(offset)?;
            let word_size = self.variables.borrow(SType::Num);
            self.mir.push(Statement::Assign(
                word_size,
                Expression::Const(Value::from(U256::from(32))).ty(SType::Num),
            ));
            let mut tmp = self.variables.borrow(SType::Num);

            for tp in &self.fun.native_output {
                self.mir.push(Statement::Assign(
                    tmp,
                    Expression::MLoad {
                        memory: self.mem_var,
                        offset,
                    }
                    .ty(SType::Num),
                ));
                let result = self.cast(tmp, SType::from_eth_type(tp, self.flags.u128_io))?;
                if result.is_num() {
                    tmp = self.variables.borrow(SType::Num);
                }
                results.push(result);
                self.mir.push(Statement::Assign(
                    offset,
                    Expression::Binary(BinaryOp::Add, offset.expr(), word_size.expr())
                        .ty(SType::Num),
                ));
            }
            self.mir.push(Statement::Result(results));
        } else {
            let offset = self.get_var(offset)?;
            let len = self.get_var(len)?;
            let result = self.variables.borrow(SType::Bytes);
            self.mir.push(Statement::Assign(
                result,
                Expression::MSlice {
                    memory: self.mem_var,
                    offset,
                    len,
                }
                .ty(SType::Bytes),
            ));
            self.mir.push(Statement::Result(vec![result]));
        }
        Ok(())
    }
}
