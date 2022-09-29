use crate::bytecode::block::InstructionBlock;
use crate::bytecode::lir::context::Context;
use crate::bytecode::lir::executor::{ExecutionResult, InstructionHandler};
use crate::bytecode::lir::ir::{Expr, Lir};
use crate::bytecode::tracing::tracer::{FlowTrace, Tracer};
use crate::{BlockId, Flags, Function, OpCode};
use anyhow::{anyhow, bail, ensure, Error};
use evm_core::Opcode;
use primitive_types::U256;
use std::collections::HashMap;

pub mod context;
pub mod executor;
pub mod ir;
pub mod stack;

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
        let block_id = BlockId::default();

        loop {
            let block = self.block(&block_id)?;
            match self.translate_block(block, &mut ir, &mut ctx)? {
                BlockResult::Jmp(_) => {
                    bail!("Jmp is not supported yet");
                }
                BlockResult::CndJmp(_, _, _) => {
                    bail!("CndJmp is not supported yet");
                }
                BlockResult::Stop => {
                    bail!("Stop is not supported yet");
                }
            }
        }

        Ok(ir)
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
                ExecutionResult::Abort(code) => {
                    ir.abort(code);
                    return Ok(BlockResult::Stop);
                }
                ExecutionResult::None => {}
                ExecutionResult::Result { offset, len } => {
                    ir.result(offset, len);
                    return Ok(BlockResult::Stop);
                }
                ExecutionResult::Stop => {
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
