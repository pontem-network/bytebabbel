use crate::bytecode::block::InstructionBlock;
use crate::bytecode::hir::context::Context;
use crate::bytecode::lir::ir::Lir;
use crate::bytecode::tracing::tracer::{FlowTrace, Tracer};
use crate::{BlockId, Flags, Function};
use anyhow::{anyhow, Error};
use primitive_types::U256;
use std::collections::HashMap;

pub mod executor;
pub mod ir;

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
    ) -> Result<Lir, Error> {
        let mut ctx = Context::new(fun, contract_address, code_size, self.flags);
        let mut ir = Lir::default();
        let block_id = BlockId::default();

        loop {
            let block = self.block(&block_id)?;
            let res = self.translate_block(block, &mut ir, &mut ctx)?;
        }

        Ok(ir)
    }

    fn translate_block(
        &self,
        block: &InstructionBlock,
        ir: &mut Lir,
        ctx: &mut Context,
    ) -> Result<(), Error> {
        for instr in block.iter() {}

        Ok(())
    }

    fn block(&self, id: &BlockId) -> Result<&InstructionBlock, Error> {
        self.contract
            .get(id)
            .ok_or_else(|| anyhow!("Block {:?} not found", id))
    }
}
