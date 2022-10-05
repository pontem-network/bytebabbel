use crate::bytecode::block::InstructionBlock;
use crate::bytecode::hir2::context::Context;
use crate::bytecode::hir2::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::hir2::ir::{Hir2, VarId, _Expr};
use crate::bytecode::loc::Loc;
use crate::bytecode::tracing::tracer::{FlowTrace, Tracer};
use crate::{BlockId, Flags, Function, OpCode};
use anyhow::{anyhow, bail, ensure, Context as ErrorContext, Error};
use primitive_types::U256;
use std::collections::{BTreeMap, HashMap};

pub mod context;
pub mod debug;
pub mod executor;
pub mod ir;
pub mod stack;
pub mod vars;

pub struct IrBuilder {
    contract: HashMap<BlockId, InstructionBlock>,
    flags: Flags,
    flow: FlowTrace,
}

impl IrBuilder {
    pub fn new(contract: HashMap<BlockId, InstructionBlock>, flags: Flags) -> Result<Self, Error> {
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
    ) -> Result<Hir2, Error> {
        let mut ctx = Context::new(fun, contract_address, code_size, self.flags);
        let mut ir = Hir2::default();
        self.translate_blocks(BlockId::default(), &mut ir, &mut ctx)?;
        Ok(ir)
    }

    fn translate_blocks(
        &self,
        start: BlockId,
        ir: &mut Hir2,
        ctx: &mut Context,
    ) -> Result<(), Error> {
        let mut block_id = start;
        loop {
            let block = self.block(&block_id)?;
            match self.translate_block(block, ir, ctx)? {
                BlockResult::Jmp(block) => {
                    if self.flow.loops.contains_key(&block) {
                        bail!("Loop detected");
                    } else {
                        block_id = block;
                    }
                }
                BlockResult::CndJmp {
                    cnd,
                    true_br,
                    false_br,
                } => {
                    let cnd = ir.assign(cnd, &mut ctx.vars);
                    self.flush_context(ctx, ir)?;
                    ir.true_brunch(&ctx.loc, ctx.loc.wrap(_Expr::Var(cnd)), true_br.into());
                    let stack = ctx.stack.clone();
                    let vars = ctx.vars.clone();
                    self.translate_blocks(false_br, ir, ctx)?;
                    ir.label(&ctx.loc, true_br.into());
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

    fn flush_context(&self, ctx: &mut Context, ir: &mut Hir2) -> Result<(), Error> {
        let stack = ctx.stack.take();
        let mut stack_dump = BTreeMap::new();

        let last_idx = stack.len() - 1;
        for (i, var) in stack.into_iter().enumerate() {
            let var_id = VarId::new_var((last_idx - i) as u32);
            if let _Expr::Var(id) = var.as_ref() {
                if var_id == *id {
                    ctx.stack.push(var);
                    continue;
                }
            }

            stack_dump.insert(var_id, var.clone());
            ctx.vars.set(var_id, var);
            ctx.stack.push(ctx.loc.wrap(_Expr::Var(var_id)));
        }
        ir.save_context(&ctx.loc, stack_dump);
        Ok(())
    }

    fn translate_block(
        &self,
        block: &InstructionBlock,
        ir: &mut Hir2,
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
        Ok(BlockResult::Jmp(BlockId::from(
            block.end + block.last().map(|i| i.size()).unwrap_or(1) as u64,
        )))
    }

    fn block(&self, id: &BlockId) -> Result<&InstructionBlock, Error> {
        self.contract
            .get(id)
            .ok_or_else(|| anyhow!("Block {:?} not found", id))
    }

    pub fn dup(&self, pops: usize, ir: &mut Hir2, ctx: &mut Context) -> Result<(), Error> {
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
    Jmp(BlockId),
    CndJmp {
        cnd: Loc<_Expr>,
        true_br: BlockId,
        false_br: BlockId,
    },
    Stop,
}
