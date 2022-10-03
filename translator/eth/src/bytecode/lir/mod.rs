use crate::bytecode::block::InstructionBlock;
use crate::bytecode::lir::context::Context;
use crate::bytecode::lir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::lir::ir::{Expr, Lir};
use crate::bytecode::tracing::tracer::{FlowTrace, Tracer};
use crate::{BlockId, Flags, Function, OpCode};
use anyhow::{anyhow, bail, ensure, Error};
use primitive_types::U256;
use std::collections::HashMap;

pub mod context;
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
        dbg!(&flow.funcs);
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
    ) -> Result<Lir, Error> {
        let mut ctx = Context::new(fun, contract_address, code_size, self.flags);
        let mut ir = Lir::default();
        self.translate_blocks(BlockId::default(), &mut ir, &mut ctx)?;
        Ok(ir)
    }

    fn translate_blocks(
        &self,
        start: BlockId,
        ir: &mut Lir,
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
                BlockResult::CndJmp(cmd, _, _) => {
                    self.flush_context(ctx, ir)?;
                    println!("CndJmp: {:?}", cmd);
                    println!("ir: {:?}", ir);
                    println!("ctx: {:?}", ctx.stack);

                    bail!("CndJmp is not supported yet");
                }
                BlockResult::Stop => {
                    bail!("Stop is not supported yet");
                }
            }
        }
    }

    fn flush_context(&self, ctx: &mut Context, ir: &mut Lir) -> Result<(), Error> {
        Ok(())
    }

    fn translate_block(
        &self,
        block: &InstructionBlock,
        ir: &mut Lir,
        ctx: &mut Context,
    ) -> Result<BlockResult, Error> {
        for inst in block.iter() {
            let pops = inst.pops();
            if let OpCode::Swap(_) = inst.1 {
                ctx.stack.swap(pops);
                continue;
            }
            if let OpCode::Dup(_) = inst.1 {
                ctx.stack.dup(pops);
                continue;
            }

            let args = ctx.stack.pop(pops);
            ensure!(pops == args.len(), "Invalid stake state.");

            let result = inst.handle(args, ir, ctx);
            match result {
                ExecutionResult::Output(output) => {
                    ctx.stack.push(output);
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
                    return Ok(BlockResult::CndJmp(cnd, true_br, false_br));
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
}

pub enum BlockResult {
    Jmp(BlockId),
    CndJmp(Expr, BlockId, BlockId),
    Stop,
}
