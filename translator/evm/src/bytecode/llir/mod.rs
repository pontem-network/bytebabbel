pub mod context;
pub mod executor;
pub mod ir;
pub mod mem;
mod ops;
pub mod stack;

use crate::bytecode::block::InstructionBlock;
use crate::bytecode::flow_graph::Flow;
use crate::bytecode::llir::context::Context;
use crate::bytecode::llir::ir::Ir;
use crate::bytecode::types::Function;
use crate::BlockId;
use anyhow::{anyhow, ensure, Error};
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
        for inst in block.iter() {
            let pops = inst.pops();
            let pushes = inst.pushes();
            let mut params = ctx.pop_stack(pops);
            ensure!(pops == params.len(), "Invalid stake state.");
            executor::execute(inst, params, ctx);
        }

        todo!()
    }

    // fn exec_instruction(
    //     &mut self,
    //     inst: &Instruction,
    //     env: &Env,
    //     next_block: BlockId,
    // ) -> Result<Option<ExecResult>, Error> {
    //     log::trace!("{}", inst);
    //     let pops = inst.pops();
    //     let mut input = self.stack.pop(pops);
    //     ensure!(pops == input.len(), "Invalid stake state.");
    //     let mut ctx = Context {
    //         executor: self,
    //         input: &mut input,
    //         env,
    //         next_block,
    //         result: None,
    //     };
    //     let pushes = execute(inst, &mut ctx)?;
    //     let result = ctx.result.take();
    //     ensure!(pushes.len() == inst.pushes(), "Invalid stake state.");
    //     self.print_stack(&input, &pushes);
    //     self.stack.push(pushes);
    //     Ok(result)
    // }
}
