use crate::bytecode::hir::ir::instruction::Instruction;
use crate::bytecode::hir::ir::var::{Eval, VarId, Vars};
use crate::bytecode::mir::ir::expression::Expression;
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::{LocalIndex, SType, Value};
use crate::bytecode::mir::ir::Mir;
use crate::bytecode::mir::translation::variables::{Variable, Variables};
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
    pub(super) signer_index: LocalIndex,
    pub(super) args_index: LocalIndex,
}

impl<'a> MirTranslator<'a> {
    pub fn new(fun: &'a Function) -> MirTranslator<'a> {
        // Now we use static parameters signer and args
        // Signer = 0
        // Args = 1
        // todo Replace with dynamic parameters
        let signer = (0, SType::Address);
        let args = (1, SType::Bytes);

        let mut variables = Variables::new(vec![signer.1, args.1]);
        let mut mir = Mir::default();

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
            signer_index: signer.0,
            args_index: args.0,
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
                Instruction::Log {
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
                    self.mir.add_statement(Statement::Log {
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
        let var = vars.take(id)?;
        match var {
            Eval::Val(val) => {
                let var = self.variables.borrow(SType::Num);
                self.mapping.insert(id, var);

                self.mir.add_statement(Statement::CreateVar(
                    var,
                    Expression::Const(Value::from(val)),
                ));
            }
            Eval::UnaryOp(cmd, op) => {
                self.translate_unary_op(cmd, op, id)?;
            }
            Eval::BinaryOp(cmd, op1, op2) => {
                self.translate_binary_op(cmd, op1, op2, id)?;
            }
            Eval::TernaryOp(cmd, op1, op2, op3) => {
                self.translate_ternary_op(cmd, op1, op2, op3, id)?;
            }
            Eval::MLoad(addr) => {
                let result = self.variables.borrow(SType::Num);
                let addr = self.get_var(addr)?;
                ensure!(addr.s_type() == SType::Num, "address must be of type u128");
                self.mapping.insert(id, result);
                self.mir.add_statement(Statement::CreateVar(
                    result,
                    Expression::MLoad {
                        memory: self.mem_var,
                        offset: addr,
                    },
                ));
            }
            Eval::SLoad(addr) => {
                let result = self.variables.borrow(SType::Num);
                self.mapping.insert(id, result);
                let addr = self.get_var(addr)?;
                ensure!(addr.s_type() == SType::Num, "address must be of type u128");

                self.mir.add_statement(Statement::CreateVar(
                    result,
                    Expression::SLoad {
                        storage: self.store_var,
                        offset: addr,
                    },
                ));
            }
            Eval::MSize => {
                let result = self.variables.borrow(SType::Num);
                self.mir.add_statement(Statement::CreateVar(
                    result,
                    Expression::MSize {
                        memory: self.mem_var,
                    },
                ));
            }
            Eval::Signer => {
                let signer = self.variables.borrow_param(self.signer_index);
                let result = self.cast(signer, SType::Num)?;
                self.mapping.insert(id, result);
            }
            Eval::ArgsSize => {
                let result = self.variables.borrow(SType::Num);
                let args = self.variables.borrow_param(self.args_index);
                ensure!(args.s_type() == SType::Bytes, "args must be of type bytes");
                self.mir
                    .add_statement(Statement::CreateVar(result, Expression::BytesLen(args)));
                self.mapping.insert(id, result);
            }
            Eval::Args(offset) => {
                let result = self.variables.borrow(SType::Num);
                let data = self.variables.borrow_param(self.args_index);
                let offset = self.get_var(offset)?;
                ensure!(offset.s_type() == SType::Num, "offset must be of type num");
                ensure!(data.s_type() == SType::Bytes, "args must be of type bytes");

                self.mir.add_statement(Statement::CreateVar(
                    result,
                    Expression::ReadNum { data, offset },
                ));
                self.mapping.insert(id, result);
            }
            Eval::Hash(offset, len) => {
                let result = self.variables.borrow(SType::Num);

                let offset = self.get_var(offset)?;
                let len = self.get_var(len)?;
                ensure!(offset.s_type() == SType::Num, "offset must be of type num");
                ensure!(len.s_type() == SType::Num, "len must be of type num");

                self.mir.add_statement(Statement::CreateVar(
                    result,
                    Expression::Hash {
                        mem: self.mem_var,
                        offset,
                        len,
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

    fn translate_ret(&mut self, offset: VarId, len: VarId) -> Result<(), Error> {
        let offset = self.get_var(offset)?;
        let len = self.get_var(len)?;
        let result = self.variables.borrow(SType::Bytes);
        self.mir.add_statement(Statement::CreateVar(
            result,
            Expression::MSlice {
                memory: self.mem_var,
                offset,
                len,
            },
        ));

        self.mir.add_statement(Statement::Result(vec![result]));
        Ok(())
    }
}
