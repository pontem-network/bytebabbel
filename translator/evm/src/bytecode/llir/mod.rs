pub mod context;
pub mod ir;

use crate::bytecode::block::InstructionBlock;
use crate::bytecode::executor::env::Env;
use crate::bytecode::executor::mem::Memory;
use crate::bytecode::executor::stack::Stack;
use crate::bytecode::flow_graph::{Flow, FlowBuilder};
use crate::bytecode::llir::context::Context;
use crate::bytecode::llir::ir::Ir;
use crate::{BlockId, Function, FunctionFlow};
use anyhow::{anyhow, Error};
use std::collections::HashMap;

pub struct Translator<'a> {
    contract: &'a HashMap<BlockId, InstructionBlock>,
    contact_flow: Flow,
}

impl<'a> Translator<'a> {
    pub fn new(contract: &'a HashMap<BlockId, InstructionBlock>, contact_flow: Flow) -> Translator {
        Translator {
            contract,
            contact_flow,
        }
    }

    pub fn translate(&self, fun: Function) -> Result<Ir, Error> {
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
