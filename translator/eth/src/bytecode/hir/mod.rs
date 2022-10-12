use crate::bytecode::block::InstructionBlock;
use crate::bytecode::hir::context::Context;
use crate::bytecode::hir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir::ir::{Expr, Label, VarId, _Expr};
use crate::bytecode::hir::vars::Vars;

use crate::bytecode::tracing::tracer::{FlowTrace, Tracer};
use crate::{Flags, Function, Hir, Offset, OpCode};
use anyhow::{anyhow, ensure, Context as ErrorContext, Error};
use primitive_types::U256;
use std::collections::{BTreeMap, HashMap};

pub mod context;
pub mod debug;
pub mod executor;
pub mod ir;
pub mod stack;
pub mod vars;

pub struct HirBuilder {
    contract: HashMap<Offset, InstructionBlock>,
    flags: Flags,
    flow: FlowTrace,
}

impl HirBuilder {
    pub fn new(contract: HashMap<Offset, InstructionBlock>, flags: Flags) -> Result<Self, Error> {
        let flow = Tracer::new(&contract).trace()?;
        Ok(Self {
            contract,
            flags,
            flow,
        })
    }

    pub fn translate_fun(
        &self,
        fun: &Function,
        contract_address: U256,
        code_size: u128,
    ) -> Result<Hir, Error> {
        let mut ctx = Context::new(fun, contract_address, code_size, self.flags);
        let mut ir = Hir::default();
        self.translate_blocks(Offset::default(), &mut ir, &mut ctx)?;
        Ok(ir)
    }

    fn translate_blocks(
        &self,
        start: Offset,
        ir: &mut Hir,
        ctx: &mut Context,
    ) -> Result<(), Error> {
        let mut block_id = start;
        loop {
            let block = self.block(&block_id)?;
            match self.translate_block(block, ir, ctx)? {
                BlockResult::Jmp(block) => {
                    if self.flow.loops.contains_key(&block) {
                        self.flush_context(ctx, ir);
                        let (from, new_loop) = ctx.create_loop(block, block_id);
                        let label = Label::new(block).from(from);
                        if new_loop {
                            ir.label(&ctx.loc, label);
                            block_id = block;
                        } else {
                            ir.goto(&ctx.loc, label);
                            return Ok(());
                        }
                    } else {
                        block_id = block;
                    }
                }
                BlockResult::CndJmp {
                    cnd,
                    true_br,
                    false_br,
                } => {
                    let jmp_id = ctx.next_jmp_id();
                    let cnd = ir.assign(cnd, &mut ctx.vars);
                    self.flush_context(ctx, ir);
                    ir.true_brunch(
                        &ctx.loc,
                        ctx.loc.wrap(_Expr::Var(cnd)),
                        Label::new(true_br).from(jmp_id),
                    );
                    let stack = ctx.stack.clone();
                    let vars = ctx.vars.clone();
                    self.translate_blocks(false_br, ir, ctx)?;
                    ir.label(&ctx.loc, Label::new(true_br).from(jmp_id));
                    ctx.stack = stack;
                    ctx.vars = vars;
                    self.translate_blocks(true_br, ir, ctx)?;
                    return Ok(());
                }
                BlockResult::Stop => {
                    return Ok(());
                }
            }
        }
    }

    fn flush_context(&self, ctx: &mut Context, ir: &mut Hir) {
        let stack = ctx.stack.take();
        let mut stack_dump = BTreeMap::new();
        let last_idx = stack.len() - 1;
        let mut vars = Vars::default();
        for (i, var) in stack.into_iter().enumerate() {
            let var_id = VarId::new_var((last_idx - i) as u32);

            let unvaried = var.unvar(ctx);
            vars.set(var_id, unvaried);

            if let _Expr::Var(id) = var.as_ref() {
                if var_id == *id {
                    ctx.stack.push(var);
                    continue;
                }
            }

            stack_dump.insert(var_id, var.clone());
            ctx.stack.push(ctx.loc.wrap(_Expr::Var(var_id)));
        }
        ctx.vars = vars;
        ir.save_stack(&ctx.loc, stack_dump);
    }

    fn translate_block(
        &self,
        block: &InstructionBlock,
        ir: &mut Hir,
        ctx: &mut Context,
    ) -> Result<BlockResult, Error> {
        for inst in block.iter() {
            let pops = inst.pops();
            ctx.loc = inst.location();
            if let OpCode::Swap(_) = inst.1 {
                ctx.stack.swap(pops);
                continue;
            }
            if let OpCode::Dup(_) = inst.1 {
                self.dup(pops, ir, ctx).context("dup stack")?;
                continue;
            }

            let args = ctx.stack.pop_vec(pops);
            ensure!(pops == args.len(), "Invalid stake state.");
            let result = inst.handle(args, ir, ctx);

            match result {
                ExecutionResult::Output(output) => {
                    ctx.stack.push(ctx.loc.wrap(output));
                }
                ExecutionResult::None => {}
                ExecutionResult::End => {
                    return Ok(BlockResult::Stop);
                }
                ExecutionResult::Jmp(br) => {
                    return Ok(BlockResult::Jmp(br));
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
            }
        }
        Ok(BlockResult::Jmp(
            block.end + block.last().map(|i| i.size()).unwrap_or(1),
        ))
    }

    fn block(&self, id: &Offset) -> Result<&InstructionBlock, Error> {
        self.contract
            .get(id)
            .ok_or_else(|| anyhow!("Block {:?} not found", id))
    }

    pub fn dup(&self, pops: usize, ir: &mut Hir, ctx: &mut Context) -> Result<(), Error> {
        let src = ctx
            .stack
            .get_mut(pops)
            .ok_or_else(|| anyhow!("Invalid stack state. "))?;
        if !src.is_var() {
            let var = ir.assign(src.clone(), &mut ctx.vars);
            *src = ctx.loc.wrap(_Expr::Var(var));
        }

        ctx.stack.dup(pops);
        let var = ctx
            .stack
            .pop()
            .ok_or_else(|| anyhow!("Invalid stack state"))?;
        ctx.stack.push(ctx.loc.wrap(_Expr::Copy(Box::new(var))));
        Ok(())
    }
}

pub enum BlockResult {
    Jmp(Offset),
    CndJmp {
        cnd: Expr,
        true_br: Offset,
        false_br: Offset,
    },
    Stop,
}
