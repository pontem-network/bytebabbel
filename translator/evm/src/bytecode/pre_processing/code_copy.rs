use crate::bytecode::block::InstructionBlock;
use crate::{BlockId, FlowBuilder, HirTranslator, OpCode};
use anyhow::Error;
use std::collections::HashMap;

pub fn find_entry_points(
    contract: &HashMap<BlockId, InstructionBlock>,
) -> Result<Option<BlockId>, Error> {
    let has_code_copy = contract
        .iter()
        .flat_map(|i| i.1.iter())
        .any(|i| i.1 == OpCode::CodeCopy);
    if !has_code_copy {
        return Ok(None);
    }

    let contract_flow = FlowBuilder::new(&contract).make_flow();
    let hir = HirTranslator::new(&contract, contract_flow);
    hir.find_entry_points()
}
