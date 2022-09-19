mod const_pool;
mod context;
mod executor;
mod ir;
mod optimizer;
mod stack;
mod vars;

use crate::bytecode::block::InstructionBlock;
use crate::bytecode::flow_graph::{Flow, IfFlow, LoopFlow};
use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir2::ir::debug::print_ir;
use crate::bytecode::hir2::ir::expression::Expr;
use crate::bytecode::hir2::ir::statement::Statement;
use crate::bytecode::hir2::ir::Hir2;
use crate::bytecode::hir2::optimizer::optimize;
use crate::bytecode::tracing::tracer::BlockIO;
use crate::{BlockId, Flags, Function};
use anyhow::{anyhow, bail, ensure, Error};
use primitive_types::U256;
use std::collections::HashMap;

pub struct HirTranslator2<'a> {
    contract: &'a HashMap<BlockId, InstructionBlock>,
    contact_flow: Flow,
    block_io: HashMap<BlockId, BlockIO>,
    flags: Flags,
}

impl<'a> HirTranslator2<'a> {
    pub fn new(
        contract: &'a HashMap<BlockId, InstructionBlock>,
        contact_flow: Flow,
        block_io: HashMap<BlockId, BlockIO>,
        flags: Flags,
    ) -> HirTranslator2 {
        HirTranslator2 {
            contract,
            contact_flow,
            block_io,
            flags,
        }
    }

    pub fn translate_fun(
        &self,
        fun: &Function,
        contract_address: U256,
        code_size: u128,
    ) -> Result<Hir2, Error> {
        let mut ctx = Context::new(fun, contract_address, code_size, self.flags);
        let mut ir = Hir2::default();
        self.exec_flow(&self.contact_flow, &mut ir, &mut ctx)?;
        print_ir(&ir, &fun.name)?;
        ir = optimize(ir, &mut ctx);
        print_ir(&ir, &fun.name)?;
        Ok(ir)
    }

    fn get_block(&self, block_id: &BlockId) -> Result<&InstructionBlock, Error> {
        self.contract
            .get(block_id)
            .ok_or_else(|| anyhow!("block not found"))
    }

    fn get_block_io(&self, block_id: &BlockId) -> Result<&BlockIO, Error> {
        self.block_io
            .get(block_id)
            .ok_or_else(|| anyhow!("block io not found"))
    }

    fn exec_flow(&self, flow: &Flow, ir: &mut Hir2, ctx: &mut Context) -> Result<StopFlag, Error> {
        match flow {
            Flow::Block(id) => self.exec_flow_block(id, ir, ctx),
            Flow::Loop(loop_) => self.exec_flow_loop(loop_, ir, ctx),
            Flow::IF(if_) => self.exec_flow_if(if_, ir, ctx),
            Flow::Sequence(seq) => self.exec_flow_seq(seq, ir, ctx),
            Flow::Continue(block) => self.exec_flow_continue(block, ir, ctx),
        }
    }

    fn exec_flow_continue(
        &self,
        block: &BlockId,
        ir: &mut Hir2,
        _ctx: &mut Context,
    ) -> Result<StopFlag, Error> {
        ir.add_statement(Statement::Continue {
            loop_id: *block,
            context: vec![],
        });
        Ok(StopFlag::Continue)
    }

    fn exec_flow_loop(
        &self,
        loop_: &LoopFlow,
        ir: &mut Hir2,
        ctx: &mut Context,
    ) -> Result<StopFlag, Error> {
        ctx.enter_loop();
        let mut cnd_ir = Hir2::default();
        let res = self.exec_block(&loop_.jmp.block, &mut cnd_ir, ctx)?;

        match res {
            BlockResult::Jmp(cnd, _) => {
                let mut loop_ir = Default::default();
                self.exec_flow(loop_.br.flow(), &mut loop_ir, &mut ctx.inherit())?;
                ctx.exit_loop();
                ir.add_statement(Statement::Loop {
                    id: loop_.jmp.block,
                    condition_block: cnd_ir.inner(),
                    condition: cnd,
                    is_true_br_loop: loop_.br.is_true_br_loop(),
                    loop_br: loop_ir.inner(),
                });
                Ok(StopFlag::Continue)
            }
            BlockResult::CndJmp {
                cnd,
                true_br,
                false_br,
            } => {
                ensure!(true_br == loop_.jmp.true_br, "invalid true_br");
                ensure!(false_br == loop_.jmp.false_br, "invalid false_br");

                let mut loop_ir = Default::default();
                self.exec_flow(loop_.br.flow(), &mut loop_ir, &mut ctx.inherit())?;
                //todo ctx.exit_loop();

                ir.add_statement(Statement::Loop {
                    id: loop_.jmp.block,
                    condition_block: cnd_ir.inner(),
                    condition: cnd,
                    is_true_br_loop: loop_.br.is_true_br_loop(),
                    loop_br: loop_ir.inner(),
                });
                Ok(StopFlag::Continue)
            }
            _ => bail!("loop condition must be a conditional jump"),
        }
    }

    fn exec_flow_seq(
        &self,
        seq: &[Flow],
        ir: &mut Hir2,
        ctx: &mut Context,
    ) -> Result<StopFlag, Error> {
        for flow in seq {
            if let StopFlag::Stop = self.exec_flow(flow, ir, ctx)? {
                return Ok(StopFlag::Stop);
            }
        }
        Ok(StopFlag::Continue)
    }

    fn exec_flow_block(
        &self,
        id: &BlockId,
        ir: &mut Hir2,
        ctx: &mut Context,
    ) -> Result<StopFlag, Error> {
        let res = self.exec_block(id, ir, ctx)?;
        match res {
            BlockResult::Jmp(_, _) => {
                //no op
                Ok(StopFlag::Continue)
            }
            BlockResult::CndJmp { .. } => {
                bail!("conditional jump not supported");
            }
            BlockResult::Stop => {
                ir.add_statement(Statement::Stop);
                Ok(StopFlag::Stop)
            }
            BlockResult::Result { offset, len } => {
                ir.add_statement(Statement::Result { offset, len });
                Ok(StopFlag::Stop)
            }
            BlockResult::Abort(code) => {
                ir.add_statement(Statement::Abort(code));
                Ok(StopFlag::Stop)
            }
        }
    }

    fn exec_flow_if(
        &self,
        if_: &IfFlow,
        ir: &mut Hir2,
        ctx: &mut Context,
    ) -> Result<StopFlag, Error> {
        let cnd_block = if_.jmp.block;
        let res = self.exec_block(&cnd_block, ir, ctx)?;
        match res {
            BlockResult::Jmp(_, jmp) => {
                if jmp == if_.jmp.true_br {
                    self.exec_flow(&if_.true_br, ir, ctx)
                } else if jmp == if_.jmp.false_br {
                    self.exec_flow(&if_.false_br, ir, ctx)
                } else {
                    Err(anyhow!("invalid jmp"))
                }
            }
            BlockResult::CndJmp {
                cnd,
                true_br,
                false_br,
            } => {
                ensure!(true_br == if_.jmp.true_br, "invalid true_br");
                ensure!(false_br == if_.jmp.false_br, "invalid false_br");
                let mut true_ir = Hir2::default();
                self.exec_flow(&if_.true_br, &mut true_ir, &mut ctx.inherit())?;
                let mut false_ir = Hir2::default();
                self.exec_flow(&if_.false_br, &mut false_ir, &mut ctx.inherit())?;

                ir.add_statement(Statement::If {
                    condition: cnd,
                    true_branch: true_ir.inner(),
                    false_branch: false_ir.inner(),
                });
                Ok(StopFlag::Continue)
            }
            _ => Err(anyhow!("unexpected block result")),
        }
    }

    fn exec_block(
        &self,
        id: &BlockId,
        ir: &mut Hir2,
        ctx: &mut Context,
    ) -> Result<BlockResult, Error> {
        let block = self.get_block(id)?;
        let io = self.get_block_io(id)?;

        for inst in block.iter() {
            let pops = inst.pops();
            let params = ctx.pop_stack(pops);
            ensure!(pops == params.len(), "Invalid stake state.");
            let res = inst.handle(params, ctx);

            match res {
                ExecutionResult::Abort(code) => {
                    return Ok(BlockResult::Abort(code));
                }
                ExecutionResult::None => {}
                ExecutionResult::Expr(mut stack) => {
                    if let Some(si) = io.outputs.get(&inst.0) {
                        if si.is_positive() {
                            let expr = stack.get_mut(0).unwrap();
                            let var = ctx.push_var(expr.clone(), *si);
                            if let Some(val) = expr.val() {
                                ctx.const_pool().assign_val(var, val);
                            } else {
                                ctx.const_pool().assign_var(var);
                            }
                            ir.add_statement(Statement::Assign {
                                var,
                                expr: expr.clone(),
                            });
                            *expr = Expr::Var(var);
                        }
                    }
                    ensure!(stack.len() == inst.pushes(), "Invalid stake state.");
                    ctx.push_stack(stack);
                }
                ExecutionResult::Result { offset, len } => {
                    return Ok(BlockResult::Result { offset, len });
                }
                ExecutionResult::Stop => {
                    return Ok(BlockResult::Stop);
                }
                ExecutionResult::Jmp(cnd, block) => {
                    return Ok(BlockResult::Jmp(cnd, block));
                }
                ExecutionResult::CndJmp {
                    cnd,
                    true_br,
                    false_br,
                } => {
                    return Ok(BlockResult::CndJmp {
                        cnd,
                        true_br,
                        false_br,
                    });
                }
                ExecutionResult::Statement(st) => {
                    ir.add_statement(st);
                }
            }
        }
        Ok(BlockResult::Jmp(
            Expr::Val(U256::zero()),
            block
                .last()
                .map(|i| BlockId::from(i.next()))
                .unwrap_or_default(),
        ))
    }
}

#[derive(Debug)]
pub enum StopFlag {
    Continue,
    Stop,
}

#[derive(Debug)]
pub enum BlockResult {
    Jmp(Expr, BlockId),
    CndJmp {
        cnd: Expr,
        true_br: BlockId,
        false_br: BlockId,
    },
    Stop,
    Result {
        offset: Expr,
        len: Expr,
    },
    Abort(u8),
}
