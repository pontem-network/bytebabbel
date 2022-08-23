use crate::bytecode::block::InstructionBlock;
use crate::bytecode::tracing::exec::{Executor, Next};
use crate::bytecode::tracing::flow::Flow;
use crate::BlockId;
use anyhow::Error;
use std::collections::{HashMap, HashSet};

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
        let loop_ = self.clone().find_loops();
        println!("++++++++++++++++++++++++++++++++++++++++++++++++");
        for (id, lp) in loop_ {
            println!("{:?}", id);
            println!("{:?}", lp);
        }
        println!("\n\n\n");
        // let block = self.blocks.get(&block);
        // match block {
        //     Some(block) => self.executor.exec_block(block),
        //     None => Next::Stop,
        // }
        todo!();
    }

    pub fn trace(&mut self, id: BlockId) -> Result<Flow, Error> {
        let mut flow = vec![];
        let loops = self.clone().find_loops();
        todo!();

        let block = self
            .blocks
            .get(&id)
            .ok_or_else(|| anyhow::anyhow!("block {} not found", id))?;
        let res = self.executor.exec_block(block);

        match res {
            Next::Jmp(jmp_to) => {
                let mut loop_detector = self.clone();
                let loop_ = loop_detector.find_loops();
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

    fn find_loops(&mut self) -> HashMap<BlockId, Loop> {
        let mut id = BlockId::default();
        let mut stack: Vec<Fork> = vec![];
        let mut loop_candidates: HashMap<BlockId, (BlockId, Vec<BlockId>)> = HashMap::new();
        let mut loops: HashMap<BlockId, Loop> = HashMap::new();
        let mut breaks: HashMap<BlockId, BlockId> = HashMap::new();
        loop {
            let block = self.blocks.get(&id).unwrap();

            let res = self.executor.exec_block(block);
            match res {
                Next::Jmp(jmp) => {
                    if let Some(lp) = breaks.get(&jmp) {
                        loops.get_mut(&lp).unwrap().breaks.insert(id);
                    }

                    if let Some(lp) = loops.get_mut(&jmp) {
                        id = lp.loop_exit;
                        continue;
                    }

                    let loop_fork = stack.iter().find(|f| f.id == jmp);
                    if let Some(fork) = loop_fork {
                        if !loop_candidates.contains_key(&fork.id) {
                            loop_candidates.entry(fork.id).or_insert((
                                fork.id,
                                self.executor.path[fork.exec.path.len()..].to_vec(),
                            ));
                        } else {
                            let (id, candidate) = loop_candidates.get(&fork.id).unwrap();
                            let loop_blocks = &self.executor.path[fork.exec.path.len()..];
                            if loop_blocks.len() > candidate.len() + 1 {
                                let lp = &loop_blocks[candidate.len() + 1..];
                                let lp2 = &loop_blocks[..candidate.len()];
                                if lp == lp2 && lp == candidate {
                                    let (loop_br, loop_exit) = if lp[0] == fork.state.0 {
                                        fork.state
                                    } else {
                                        (fork.state.1, fork.state.0)
                                    };
                                    loops.insert(
                                        *id,
                                        Loop {
                                            root: *id,
                                            loop_exit,
                                            loop_br,
                                            continuous: lp.last().unwrap().clone(),
                                            breaks: HashSet::new(),
                                            fork: fork.clone(),
                                        },
                                    );
                                    breaks.insert(loop_exit, *id);
                                    loop_candidates.clear();
                                }
                            }
                        }
                    }
                    id = jmp;
                }
                Next::Stop => loop {
                    if let Some(fork) = stack.pop() {
                        self.executor = fork.exec.clone();
                        if let Some(false_br) = fork.next_br {
                            id = false_br;
                            stack.push(Fork {
                                id: fork.id,
                                exec: fork.exec,
                                state: fork.state,
                                next_br: None,
                            });
                            break;
                        }
                    } else {
                        return loops;
                    }
                },
                Next::Cnd(true_br, false_br) => {
                    if let Some(lp) = breaks.get(&true_br) {
                        loops.get_mut(&lp).unwrap().breaks.insert(id);
                        stack.push(Fork {
                            id,
                            exec: self.executor.clone(),
                            state: (true_br, false_br),
                            next_br: None,
                        });
                        id = false_br;
                        continue;
                    }

                    if let Some(lp) = breaks.get(&false_br) {
                        loops.get_mut(&lp).unwrap().breaks.insert(id);
                    }

                    stack.push(Fork {
                        id,
                        exec: self.executor.clone(),
                        state: (true_br, false_br),
                        next_br: Some(false_br),
                    });
                    id = true_br;
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum LoopCandidate {
    Candidate(BlockId, Vec<BlockId>),
}

#[derive(Clone, Debug)]
pub struct Fork {
    pub id: BlockId,
    pub exec: Executor,
    pub state: (BlockId, BlockId),
    pub next_br: Option<BlockId>,
}

#[derive(Debug)]
pub struct Loop {
    pub root: BlockId,
    pub loop_exit: BlockId,
    pub loop_br: BlockId,
    pub continuous: BlockId,
    pub breaks: HashSet<BlockId>,
    pub fork: Fork,
}
