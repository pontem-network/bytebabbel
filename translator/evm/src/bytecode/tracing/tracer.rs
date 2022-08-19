use crate::bytecode::block::InstructionBlock;
use crate::bytecode::tracing::exec::{Executor, Next};
use crate::bytecode::tracing::flow::Flow;
use crate::BlockId;
use anyhow::Error;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Tracer<'a> {
    blocks: &'a HashMap<BlockId, InstructionBlock>,
    executor: Executor,
}

impl<'a> Tracer<'a> {
    pub fn new(blocks: &'a HashMap<BlockId, InstructionBlock>, executor: Executor) -> Self {
        Self { blocks, executor }
    }

    pub fn exec(&mut self, block: BlockId) -> Next {
        let block = self.blocks.get(&block);
        match block {
            Some(block) => self.executor.exec_block(block),
            None => Next::Stop,
        }
    }

    pub fn trace(&mut self, id: BlockId) -> Result<Flow, Error> {
        let mut flow = vec![];
        let block = self
            .blocks
            .get(&id)
            .ok_or_else(|| anyhow::anyhow!("block {} not found", id))?;
        let res = self.executor.exec_block(block);

        match res {
            Next::Jmp(jmp_to) => {
                let mut loop_detector = self.clone();
                let loop_ = loop_detector.find_loop(id);
                println!("loop: {:?}", loop_);
                // let stop_block = block
                //     .last()
                //     .map(|lst| BlockId::from(lst.offset() + lst.size()))
                //     .unwrap_or(BlockId::default());
                //
                // let mut tracer = self.clone();
                // let sec = tracer.trace(jmp_to)?;
                // println!("seq: {:?}", sec);
            }
            Next::Stop => {
                flow.push(Flow::Block(id));
                flow.push(Flow::Stop);
            }
            Next::Cnd(true_br, false_br) => {
                flow.push(Flow::If {
                    cnd: id,
                    true_br: Box::new(self.clone().trace(true_br)?),
                    false_br: Box::new(self.clone().trace(false_br)?),
                });
            }
        }

        Ok(Flow::Sequence(flow))
    }

    fn find_loop(&mut self, loop_head: BlockId) -> Option<Loop> {
        let mut id = loop_head;
        let mut stack: Vec<Fork> = vec![];
        let mut depth = 1;
        let loops: Vec<BlockId> = vec![];
        println!("loop_head: {:?}", loop_head);
        loop {
            if depth == 1000 {
                panic!();
            } else {
                depth += 1;
            }

            let block = self.blocks.get(&id)?;

            let res = self.executor.exec_block(block);
            match res {
                Next::Jmp(jmp) => {
                    id = jmp;
                }
                Next::Stop => loop {
                    if let Some(fork) = stack.pop() {
                        self.executor = fork.exec.clone();
                        if let Some(false_br) = fork.false_br {
                            id = false_br;
                            stack.push(Fork {
                                id,
                                exec: fork.exec,
                                false_br: None,
                            });
                            break;
                        }
                    } else {
                        return None;
                    }
                },
                Next::Cnd(true_br, false_br) => {
                    println!("id: {:?}", id);
                    println!("true_br: {:?}", true_br);
                    println!("false_br: {:?}", false_br);
                    stack.push(Fork {
                        id,
                        exec: self.executor.clone(),
                        false_br: Some(false_br),
                    });
                    id = true_br;
                }
            }
        }
    }
}

pub struct Fork {
    pub id: BlockId,
    pub exec: Executor,
    pub false_br: Option<BlockId>,
}

#[derive(Debug)]
pub struct Loop {
    pub root: BlockId,
    pub continues: Vec<BlockId>,
    pub breaks: Vec<BlockId>,
}
