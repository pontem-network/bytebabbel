use std::collections::{BTreeMap, HashMap};

use anyhow::{anyhow, Error};
use primitive_types::U256;

use crate::bytecode::hir::executor::math::BinaryOp;
use crate::bytecode::hir::ir::{Stmt, VarId, _Expr};
use crate::bytecode::hir::vars::Vars;
use crate::bytecode::loc::Loc;
use crate::bytecode::mir::ir::expression::Expression;
use crate::bytecode::mir::ir::statement::Statement;
use crate::bytecode::mir::ir::types::{LocalIndex, SType, Value};
use crate::bytecode::mir::ir::Mir;
use crate::bytecode::mir::translation::variables::{Variable, Variables};
use crate::{Flags, Function, Hir};

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
    pub(super) vars: Variables,
    pub(super) var_map: HashMap<VarId, Variable>,
    pub(super) ctx_map: HashMap<VarId, Variable>,
    pub(super) mir: Mir,
    pub(super) mem_var: Variable,
    pub(super) store_var: Variable,
    pub(super) signer_index: LocalIndex,
    pub(super) args_index: LocalIndex,
    pub(super) flags: Flags,
    pub(super) loc: Loc<()>,
}

impl<'a> MirTranslator<'a> {
    pub fn new(fun: &'a Function, flags: Flags) -> MirTranslator<'a> {
        let loc = Loc::default();
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

        let store_var = variables.borrow(SType::Storage);
        mir.push(loc.wrap(store_var.assign(Expression::GetStore.ty(SType::Storage))));

        let mem_var = variables.borrow(SType::Memory);
        mir.push(loc.wrap(mem_var.assign(Expression::GetMem.ty(SType::Memory))));

        MirTranslator {
            fun,
            vars: variables,
            var_map: Default::default(),
            ctx_map: Default::default(),
            mir,
            mem_var,
            store_var,
            signer_index: signer.0,
            args_index: args.0,
            flags,
            loc,
        }
    }

    fn prepare_context_vars(&mut self, hir: &Hir) {
        let instructions = hir.statements();
        let ctx = instructions
            .iter()
            .map(|inst| {
                if let Stmt::StoreContext(ctx) = inst {
                    ctx.len()
                } else {
                    0
                }
            })
            .fold(0, |acc, x| if x > acc { x } else { acc });

        for idx in 0..ctx {
            let var_id = VarId::new_var(idx as u32);
            let var = self.vars.borrow(SType::Num);
            self.ctx_map.insert(var_id, var);
            self.vars.release(var);
        }
    }

    pub fn translate(mut self, hir: Hir) -> Result<Mir, Error> {
        self.prepare_context_vars(&hir);
        let instructions = hir.inner();
        self.translate_instructions(instructions)?;
        self.mir.set_locals(self.vars.locals());
        Ok(self.mir)
    }

    fn translate_instructions(&mut self, stmts: Vec<Loc<Stmt>>) -> Result<(), Error> {
        for stmt in stmts {
            self.loc = stmt.wrap(());

            match stmt.inner() {
                Stmt::Label(label) => {
                    self.mir.push(self.loc.wrap(Statement::Label(*label)));
                }
                Stmt::StoreContext(ctx) => {
                    self.translate_store_context(ctx)?;
                }
                Stmt::Assign(var, expr) => {
                    self.translate_set_var(*var, expr)?;
                }
                Stmt::MemStore8 { addr, val } => {
                    self.translate_mem_store8(addr, val)?;
                }
                Stmt::MemStore { addr, val } => {
                    self.translate_mem_store(addr, val)?;
                }
                Stmt::SStore { key, val } => {
                    self.translate_s_store(key, val)?;
                }
                Stmt::Log {
                    offset,
                    len,
                    topics,
                } => {
                    self.translate_log(offset, len, topics)?;
                }
                Stmt::Stop => {
                    if !self.fun.native_output.is_empty() {
                        self.mir.push(self.loc.wrap(Statement::Abort(u8::MAX)));
                    } else {
                        self.translate_ret_unit()?;
                    }
                }
                Stmt::Abort(code) => {
                    self.mir.push(self.loc.wrap(Statement::Abort(*code)));
                }
                Stmt::Result { offset, len } => {
                    self.translate_ret(offset, len)?;
                }
                Stmt::BrunchTrue(cnd, label) => {}
                Stmt::Brunch(label) => {}
            }
            // match stmt.as_ref() {
            //     HirStmt::SetVar(id) => {
            //         self.translate_set_var(*id, vars)?;
            //     }
            //     HirStmt::MemStore { addr, var } => {
            //         self.translate_mem_store(*addr, *var, vars)?;
            //     }
            //     HirStmt::MemStore8 { addr, var } => {
            //         self.translate_mem_store8(*addr, *var, vars)?;
            //     }
            //     HirStmt::SStore { addr, var } => {
            //         self.translate_s_store(*addr, *var, vars)?;
            //     }
            //     HirStmt::If {
            //         condition,
            //         true_branch,
            //         false_branch,
            //     } => {
            //         self.translate_if(*condition, true_branch, false_branch, vars)?;
            //     }
            //     HirStmt::Loop {
            //         id,
            //         condition_block,
            //         condition,
            //         is_true_br_loop,
            //         loop_br,
            //     } => {
            //         self.translate_loop(
            //             *id,
            //             condition_block,
            //             *condition,
            //             *is_true_br_loop,
            //             loop_br,
            //             vars,
            //         )?;
            //     }
            //     HirStmt::Stop => {
            //         if !self.fun.native_output.is_empty() {
            //             self.mir.push(Statement::Abort(u8::MAX));
            //         } else {
            //             self.translate_ret_unit()?;
            //         }
            //     }
            //     HirStmt::Abort(code) => {
            //         self.mir.push(Statement::Abort(*code));
            //     }
            //     HirStmt::Result { offset, len } => {
            //         self.translate_ret(*offset, *len)?;
            //     }
            //     HirStmt::MapVar { id, val } => {
            //         let val = self.get_var(*val)?;
            //         let id = self.get_var(*id)?;
            //         self.mir.push(Statement::Assign(id, val.expr()));
            //     }
            //     HirStmt::Continue {
            //         loop_id: id,
            //         context: inst,
            //     } => {
            //         self.translate_instructions(inst, vars)?;
            //         self.mir.push(Statement::Continue(*id));
            //     }
            //     HirStmt::Log {
            //         offset,
            //         len,
            //         topics,
            //     } => {
            //         let topics = topics
            //             .iter()
            //             .map(|t| self.get_var(*t))
            //             .collect::<Result<Vec<_>, _>>()?;
            //
            //         let offset = self.get_var(*offset)?;
            //         let len = self.get_var(*len)?;
            //         self.mir.push(Statement::Log {
            //             storage: self.store_var,
            //             memory: self.mem_var,
            //             offset,
            //             len,
            //             topics,
            //         });
            //     }
            // }
        }
        Ok(())
    }

    fn translate_store_context(&mut self, ctx: BTreeMap<VarId, Loc<_Expr>>) -> Result<(), Error> {
        let mut context = BTreeMap::new();
        for (var, expr) in ctx {
            let expr = self.translate_expr(expr)?;
            let expr = self.cast_expr(expr, SType::Num)?;
            let var = self
                .ctx_map
                .get(&var)
                .ok_or_else(|| anyhow!("Unknown context variable:{}", var))?;
            self.vars.reborrow(var);
            context.insert(*var, expr);
        }
        self.mir
            .push(self.loc.wrap(Statement::StoreContext(context)));
        Ok(())
    }

    fn translate_set_var(&mut self, id: VarId, expr: Loc<_Expr>) -> Result<(), Error> {
        let expr = self.translate_expr(expr)?;
        if id.is_tmp() {
            let var = self.vars.borrow(expr.ty);
            self.var_map.insert(id, var);
            self.mir.push(self.loc.wrap(Statement::Assign(var, expr)));
        } else {
            let var = *self
                .ctx_map
                .get(&id)
                .ok_or_else(|| anyhow!("Unknown context variable: {} in {}", id, self.fun.name))?;
            let expr = self.cast_expr(expr, var.ty())?;
            self.mir.push(self.loc.wrap(Statement::Assign(var, expr)));
        }
        Ok(())
    }

    fn translate_log(
        &mut self,
        offset: Loc<_Expr>,
        len: Loc<_Expr>,
        topics: Vec<Loc<_Expr>>,
    ) -> Result<(), Error> {
        let topics = topics
            .into_iter()
            .map(|t| self.translate_expr(t))
            .collect::<Result<Vec<_>, _>>()?;

        let offset = self.translate_expr(offset)?;
        let len = self.translate_expr(len)?;
        self.mir.push(self.loc.wrap(Statement::Log {
            storage: self.store_var,
            memory: self.mem_var,
            offset,
            len,
            topics,
        }));
        Ok(())
    }

    // fn get_var(&mut self, id: VarId) -> Result<Variable, Error> {
    //     let var = self
    //         .var_map
    //         .get(&id)
    //         .ok_or_else(|| anyhow!("variable {:?} not found", id))?;
    //     Ok(*var)
    // }
    //
    fn translate_ret_unit(&mut self) -> Result<(), Error> {
        if self.flags.hidden_output || self.flags.native_output {
            self.mir.push(self.loc.wrap(Statement::Result(vec![])));
            return Ok(());
        }

        let unit = self.vars.borrow(SType::Bytes);
        let len = self.vars.borrow(SType::Num);
        self.mir.push(
            self.loc.wrap(Statement::Assign(
                len,
                self.loc
                    .wrap(Expression::Const(Value::from(U256::zero())).ty(SType::Num)),
            )),
        );
        self.mir.push(
            self.loc.wrap(Statement::Assign(
                unit,
                self.loc.wrap(
                    Expression::MSlice {
                        memory: self.mem_var,
                        offset: self.loc.wrap(Expression::MoveVar(len).ty(SType::Num)),
                        len: self.loc.wrap(Expression::MoveVar(len).ty(SType::Num)),
                    }
                    .ty(SType::Bytes),
                ),
            )),
        );

        self.mir.push(self.loc.wrap(Statement::Result(vec![unit])));
        Ok(())
    }

    fn translate_ret(&mut self, offset: Loc<_Expr>, len: Loc<_Expr>) -> Result<(), Error> {
        if self.flags.hidden_output {
            self.mir.push(self.loc.wrap(Statement::Result(vec![])));
            return Ok(());
        }

        if self.flags.native_output {
            let offset = self.translate_expr(offset)?;
            let offset_var = self.vars.borrow(SType::Num);
            self.mir
                .push(self.loc.wrap(Statement::Assign(offset_var, offset)));
            let offset = self
                .loc
                .wrap(Expression::MoveVar(offset_var).ty(SType::Num));

            let mut results = vec![];
            let word_size = self.vars.borrow(SType::Num);
            self.mir.push(
                self.loc.wrap(Statement::Assign(
                    word_size,
                    self.loc
                        .wrap(Expression::Const(Value::from(U256::from(32))).ty(SType::Num)),
                )),
            );
            let word_size = self.loc.wrap(Expression::MoveVar(word_size).ty(SType::Num));

            let mut tmp = self.vars.borrow(SType::Num);
            for tp in &self.fun.native_output {
                self.mir.push(
                    self.loc.wrap(Statement::Assign(
                        tmp,
                        self.loc.wrap(
                            Expression::MLoad {
                                memory: self.mem_var,
                                offset: offset.clone(),
                            }
                            .ty(SType::Num),
                        ),
                    )),
                );

                let result = self.cast_expr(
                    self.loc.wrap(tmp.expr()),
                    SType::from_eth_type(tp, self.flags.u128_io),
                )?;
                if result.is_num() {
                    tmp = self.vars.borrow(SType::Num);
                }
                let result_var = self.vars.borrow(result.ty);
                self.mir.push(self.loc.wrap(result_var.assign(result)));

                results.push(result_var);
                self.mir.push(
                    self.loc.wrap(Statement::Assign(
                        offset_var,
                        self.loc.wrap(
                            Expression::Binary(BinaryOp::Add, offset.clone(), word_size.clone())
                                .ty(SType::Num),
                        ),
                    )),
                );
            }

            self.mir.push(self.loc.wrap(Statement::Result(results)));
        } else {
            let offset = self.translate_expr(offset)?;
            let len = self.translate_expr(len)?;
            let result = self.vars.borrow(SType::Bytes);
            self.mir.push(
                self.loc.wrap(Statement::Assign(
                    result,
                    self.loc.wrap(
                        Expression::MSlice {
                            memory: self.mem_var,
                            offset,
                            len,
                        }
                        .ty(SType::Bytes),
                    ),
                )),
            );
            self.mir
                .push(self.loc.wrap(Statement::Result(vec![result])));
        }
        Ok(())
    }
}
