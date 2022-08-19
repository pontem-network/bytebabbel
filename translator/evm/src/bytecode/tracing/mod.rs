pub mod debug;
pub mod exec;
pub mod flow;
pub mod tracer;

use crate::bytecode::block::InstructionBlock;
use crate::bytecode::tracing::exec::{Executor, Next};
use crate::bytecode::tracing::flow::Flow;
use crate::{BlockId, Tracer};
use anyhow::Error;
use std::collections::HashMap;

pub struct WFC<'a> {
    blocks: &'a HashMap<BlockId, InstructionBlock>,
}

impl<'a> WFC<'a> {
    pub fn new(blocks: &'a HashMap<BlockId, InstructionBlock>) -> Self {
        Self { blocks }
    }

    pub fn trace(&self) -> Result<Flow, Error> {
        let mut flow = vec![];

        let mut tracer = Tracer::new(self.blocks, Executor::default());
        let next = tracer.exec(BlockId::default());
        println!("{:?}", next);
        match next {
            Next::Jmp(_) => {}
            Next::Stop => {
                return Ok(Flow::Sequence(flow));
            }
            Next::Cnd(true_br, false_br) => {
                //if or loop
                let true_br = tracer.clone().trace(true_br)?;
                let false_br = tracer.clone().trace(false_br)?;
                println!("true_br: {:?}", true_br);
                println!("false_br: {:?}", false_br);
            }
        }

        println!("{:?}", flow);
        todo!();
        Ok(Flow::Sequence(flow))
    }
}
