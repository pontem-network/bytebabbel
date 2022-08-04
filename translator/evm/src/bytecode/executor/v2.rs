use crate::bytecode::block::InstructionBlock;
use crate::bytecode::executor::flow_graph::{Flow, FlowBuilder};
use crate::bytecode::executor::mem::Memory;
use crate::bytecode::executor::stack::Stack;
use crate::BlockId;
use std::collections::HashMap;

pub struct ExecutorV2<'a> {
    mem: Memory,
    stack: Stack,
    contract: &'a HashMap<BlockId, InstructionBlock>,
    new_code_offset: Option<BlockId>,
    flow: Flow,
}

impl<'a> ExecutorV2<'a> {
    pub fn new(contract: &'a HashMap<BlockId, InstructionBlock>) -> ExecutorV2 {
        let flow = FlowBuilder::new(contract).make_flow();

        ExecutorV2 {
            mem: Memory::default(),
            stack: Stack::default(),
            contract,
            new_code_offset: None,
            flow,
        }
    }
}
