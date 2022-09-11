mod context;
mod executor;
mod ir;
mod vars;

use crate::bytecode::block::InstructionBlock;
use crate::bytecode::flow_graph::Flow;
use crate::bytecode::hir2::ir::debug::print_ir;
use crate::bytecode::hir2::ir::Hir2;
use crate::bytecode::tracing::tracer::BlockIO;
use crate::{BlockId, Function};
use anyhow::Error;
use primitive_types::U256;
use std::collections::HashMap;

pub struct HirTranslator2<'a> {
    contract: &'a HashMap<BlockId, InstructionBlock>,
    contact_flow: Flow,
    block_io: HashMap<BlockId, BlockIO>,
}

impl<'a> HirTranslator2<'a> {
    pub fn new(
        contract: &'a HashMap<BlockId, InstructionBlock>,
        contact_flow: Flow,
        block_io: HashMap<BlockId, BlockIO>,
    ) -> HirTranslator2 {
        HirTranslator2 {
            contract,
            contact_flow,
            block_io,
        }
    }

    pub fn translate_fun(
        &self,
        fun: &Function,
        contract_address: U256,
        code_size: u128,
    ) -> Result<Hir2, Error> {
        let mut ir = Hir2::default();
        println!("Translate fun: {:?}", fun);

        print_ir(&ir, &fun.name)?;
        Ok(ir)
    }
}
