use crate::bytecode::hir2::executor::math::BinaryOp;
use crate::bytecode::hir2::ir::expression::Expr;
use crate::bytecode::hir2::ir::statement::Statement as Stmt;
use crate::bytecode::hir2::ir::Hir2;
use crate::bytecode::hir2::vars::VarId;
use crate::bytecode::mir::ir::expression::{Expression, StackOpsBuilder, TypedExpr};
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::{LocalIndex, SType, Value};
use crate::bytecode::mir::translation::variables::{Variable, Variables};
use crate::{Flags, Function, Mir};
use anyhow::{anyhow, bail, ensure, Error};
use primitive_types::U256;
use std::collections::HashMap;

mod cast;
mod math;
pub mod variables;

pub struct MirTranslator<'a> {
    pub(super) fun: &'a Function,
    pub(super) variables: Variables,
    pub(super) mapping: HashMap<VarId, Variable>,
    pub(super) mem_var: Option<Variable>,
    pub(super) store_var: Option<Variable>,
    pub(super) signer_index: LocalIndex,
    pub(super) args_index: LocalIndex,
    pub(super) flags: Flags,
}

impl<'a> MirTranslator<'a> {
    pub fn new(fun: &'a Function, flags: Flags) -> MirTranslator<'a> {
        let signer = (0, SType::Signer);
        let args = (1, SType::Bytes);

        let variables = if flags.native_input {
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

        MirTranslator {
            fun,
            variables,
            mapping: Default::default(),
            mem_var: None,
            store_var: None,
            signer_index: signer.0,
            args_index: args.0,
            flags,
        }
    }

    pub fn translate(mut self, hir: Hir2) -> Result<Mir, Error> {
        let statements = hir.inner();
        let mut mir = Mir::default();
        let store_var = self.variables.borrow_global(SType::Storage);
        mir.add_statement(store_var.assign(Expression::GetStore.ty(SType::Storage)));
        self.store_var = Some(store_var);

        let mem_var = self.variables.borrow_global(SType::Memory);
        mir.add_statement(mem_var.assign(Expression::GetMem.ty(SType::Memory)));
        self.mem_var = Some(mem_var);

        mir.extend_statements(self.translate_statements(&statements)?);
        mir.set_locals(self.variables.locals());
        Ok(mir)
    }

    fn translate_statements(&mut self, stmts: &[Stmt]) -> Result<Vec<Statement>, Error> {
        let mut statements = Vec::with_capacity(stmts.len());
        for stmt in stmts {
            match stmt {
                Stmt::Assign { var, expr } => {
                    let expr = self.expr(expr)?;
                    let mir_var = self.variables.borrow(expr.ty);
                    self.mapping.insert(*var, mir_var);
                    statements.push(mir_var.assign(expr))
                }
                Stmt::MemStore8 { addr, var } => statements.push(Statement::MStore8 {
                    memory: self.mem_var.ok_or_else(|| anyhow!("no memory var"))?,
                    offset: self.expr(addr)?,
                    val: self.expr(var)?,
                }),
                Stmt::MemStore { addr, var } => statements.push(Statement::MStore {
                    memory: self.mem_var.ok_or_else(|| anyhow!("no memory var"))?,
                    offset: self.expr(addr)?,
                    val: self.expr(var)?,
                }),
                Stmt::SStore { addr, var } => statements.push(Statement::SStore {
                    storage: self.store_var.ok_or_else(|| anyhow!("no storage var"))?,
                    key: self.expr(addr)?,
                    val: self.expr(var)?,
                }),
                Stmt::Log {
                    offset,
                    len,
                    topics,
                } => statements.push(Statement::Log {
                    storage: self.store_var.ok_or_else(|| anyhow!("no storage var"))?,
                    memory: self.mem_var.ok_or_else(|| anyhow!("no memory var"))?,
                    offset: self.expr(offset)?,
                    len: self.expr(len)?,
                    topics: topics
                        .iter()
                        .map(|t| self.expr(t))
                        .collect::<Result<Vec<_>, _>>()?,
                }),
                Stmt::If {
                    condition,
                    true_branch,
                    false_branch,
                } => {
                    let condition = self.expr(condition)?;
                    statements.push(Statement::IF {
                        cnd: self.cast(condition, SType::Bool)?,
                        true_br: self.translate_statements(true_branch)?,
                        false_br: self.translate_statements(false_branch)?,
                    })
                }
                Stmt::Loop {
                    id,
                    condition_block,
                    condition,
                    is_true_br_loop,
                    loop_br,
                } => {
                    let condition = self.expr(condition)?;
                    let condition = self.cast(condition, SType::Bool)?;
                    let condition = if *is_true_br_loop {
                        StackOpsBuilder::default()
                            .push_bool(condition)?
                            .not()?
                            .build(SType::Bool)?
                    } else {
                        condition
                    };

                    statements.push(Statement::Loop {
                        id: *id,
                        cnd_calc: self.translate_statements(condition_block)?,
                        cnd: condition,
                        body: self.translate_statements(loop_br)?,
                    })
                }
                Stmt::Continue { loop_id, context } => {
                    statements.extend(self.translate_statements(context)?);
                    statements.push(Statement::Continue(*loop_id));
                }
                Stmt::Stop => statements.push(if !self.fun.native_output.is_empty() {
                    Statement::Abort(u8::MAX)
                } else {
                    Statement::Result(vec![])
                }),
                Stmt::Abort(code) => statements.push(Statement::Abort(*code)),
                Stmt::Result { offset, len } => self.translate_ret(offset, len, &mut statements)?,
            };
        }
        Ok(statements)
    }

    fn expr(&mut self, expr: &Expr) -> Result<TypedExpr, Error> {
        Ok(match expr {
            Expr::Val(val) => Expression::Const(Value::from(*val)).ty(SType::Num),
            Expr::Var(var) => {
                let var = self.take_var(*var)?;
                Expression::Var(var).ty(var.s_type())
            }
            Expr::MLoad { mem_offset } => {
                let offset = self.expr(mem_offset)?;
                ensure!(offset.ty == SType::Num, "memory offset must be of type num");
                Expression::MLoad {
                    memory: self.mem_var.ok_or_else(|| anyhow!("no memory var"))?,
                    offset,
                }
                .ty(SType::Num)
            }
            Expr::SLoad { key } => {
                let key = self.expr(key)?;
                ensure!(key.ty == SType::Num, "storage key must be of type num");
                Expression::SLoad {
                    storage: self.store_var.ok_or_else(|| anyhow!("no storage var"))?,
                    key,
                }
                .ty(SType::Num)
            }
            Expr::Signer => {
                let signer = self.variables.borrow_param(self.signer_index);
                Expression::Var(signer).ty(SType::Signer)
            }
            Expr::MSize => Expression::MSize {
                memory: self.mem_var.ok_or_else(|| anyhow!("no memory var"))?,
            }
            .ty(SType::Num),
            Expr::ArgsSize => self.translate_args_size()?,
            Expr::Args { args_offset } => self.translate_args(args_offset)?,
            Expr::UnaryOp(cmd, op) => self.translate_unary_op(*cmd, op)?,
            Expr::BinaryOp(cmd, op1, op2) => self.translate_binary_op(*cmd, op1, op2)?,
            Expr::TernaryOp(cmd, op1, op2, op3) => {
                self.translate_ternary_op(*cmd, op1, op2, op3)?
            }
            Expr::Hash {
                mem_offset,
                mem_len,
            } => {
                let offset = self.expr(mem_offset)?;
                let len = self.expr(mem_len)?;
                ensure!(offset.ty == SType::Num, "memory offset must be of type num");
                ensure!(len.ty == SType::Num, "memory len must be of type num");
                Expression::Hash {
                    mem: self.mem_var.ok_or_else(|| anyhow!("no memory var"))?,
                    offset,
                    len,
                }
                .ty(SType::Num)
            }
        })
    }

    fn take_var(&mut self, id: VarId) -> Result<Variable, Error> {
        let var = self
            .mapping
            .remove(&id)
            .ok_or_else(|| anyhow!("variable {:?} not found", id))?;
        self.variables.release(&var);
        Ok(var)
    }

    fn translate_ret(
        &mut self,
        offset: &Expr,
        len: &Expr,
        statements: &mut Vec<Statement>,
    ) -> Result<(), Error> {
        if self.flags.hidden_output {
            statements.push(Statement::Result(vec![]));
        }

        if self.flags.native_output {
            let mut offset = self.expr(offset)?;
            let word_size = self.variables.borrow(SType::Num);
            statements.push(
                word_size.assign(Expression::Const(Value::from(U256::from(32))).ty(SType::Num)),
            );
            let word_size = Expression::Var(word_size).ty(SType::Num);

            let mut results = vec![];
            for tp in &self.fun.native_output {
                let offset_var = self.variables.borrow(SType::Num);
                statements.push(offset_var.assign(offset.clone()));
                let offset_var = Expression::Var(offset_var).ty(SType::Num);

                let mem_frame = Expression::MLoad {
                    memory: self.mem_var.ok_or_else(|| anyhow!("no memory var"))?,
                    offset: offset_var.clone(),
                }
                .ty(SType::Num);
                results.push(self.cast(mem_frame, SType::from_eth_type(tp, self.flags.u128_io))?);

                offset = BinaryOp::Add.expr(offset_var.clone(), word_size.clone());
            }
            statements.push(Statement::Result(results));
        } else {
            let slice = Expression::MSlice {
                memory: self.mem_var.ok_or_else(|| anyhow!("no memory var"))?,
                offset: self.expr(offset)?,
                len: self.expr(len)?,
            }
            .ty(SType::Bytes);
            statements.push(Statement::Result(vec![slice]));
        }
        Ok(())
    }

    fn translate_args_size(&mut self) -> Result<TypedExpr, Error> {
        if self.flags.native_input {
            bail!("args_size is not supported in native input mode");
        } else {
            let args = self.variables.borrow_param(self.args_index);
            ensure!(args.s_type() == SType::Bytes, "args must be of type bytes");
            Ok(Expression::BytesLen(args).ty(SType::Num))
        }
    }

    fn translate_args(&mut self, offset: &Box<Expr>) -> Result<TypedExpr, Error> {
        Ok(if self.flags.native_input {
            let offset = offset
                .resolve(None)
                .ok_or_else(|| anyhow!("args offset must be a static"))?;
            let param = self.variables.borrow_param(offset.as_u128() as LocalIndex);
            Expression::Var(param).ty(param.s_type())
        } else {
            let data = self.variables.borrow_param(self.args_index);
            let offset = self.expr(offset)?;
            ensure!(offset.ty == SType::Num, "offset must be of type num");
            ensure!(data.s_type() == SType::Bytes, "args must be of type bytes");
            Expression::ReadNum { data, offset }.ty(SType::Num)
        })
    }
}
