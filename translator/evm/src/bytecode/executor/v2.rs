mod context;
mod ir;

use crate::bytecode::block::InstructionBlock;
use crate::bytecode::executor::env::Env;
use crate::bytecode::executor::mem::Memory;
use crate::bytecode::executor::stack::Stack;
use crate::bytecode::executor::v2::context::Context;
use crate::bytecode::flow_graph::{Flow, FlowBuilder};
use crate::{BlockId, Function, FunctionFlow};
use anyhow::{anyhow, Error};
use ir::Ir;
use std::collections::HashMap;

pub struct LlIrTranslator<'a> {
    contract: &'a HashMap<BlockId, InstructionBlock>,
    contact_flow: Flow,
}

impl<'a> LlIrTranslator<'a> {
    pub fn new(contract: &'a HashMap<BlockId, InstructionBlock>) -> LlIrTranslator {
        let flow = FlowBuilder::new(contract).make_flow();
        LlIrTranslator {
            contract,
            contact_flow: flow,
        }
    }

    pub fn make_ir(&self, fun: Function) -> Result<Ir, Error> {
        let mut ctx = Context::new(fun);
        self.exec_flow(&self.contact_flow, &mut ctx)
    }

    fn get_block(&self, block_id: &BlockId) -> Result<&InstructionBlock, Error> {
        self.contract
            .get(&block_id)
            .ok_or_else(|| anyhow!("block not found"))
    }

    fn exec_flow(&self, flow: &Flow, ctx: &mut Context) -> Result<Ir, Error> {
        match flow {
            Flow::Block(id) => {
                self.exec_block(id, ctx)?;
            }
            Flow::Loop(loop_) => {}
            Flow::IF(if_) => {}
            Flow::Sequence(seq_) => {}
        }
        todo!()
    }

    fn exec_block(&self, id: &BlockId, ctx: &mut Context) -> Result<Ir, Error> {
        let block = self.get_block(&id)?;

        todo!()
    }
}
