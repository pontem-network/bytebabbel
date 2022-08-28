use crate::bytecode::block::InstructionBlock;
use crate::bytecode::tracing::exec::{Executor, Next};
use crate::{BlockId, OpCode, U256};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct Tracer<'a> {
    blocks: &'a HashMap<BlockId, InstructionBlock>,
    executor: Executor,
}

impl<'a> Tracer<'a> {
    pub fn new(blocks: &'a HashMap<BlockId, InstructionBlock>) -> Self {
        Self {
            blocks,
            executor: Executor::default(),
        }
    }

    pub fn trace(&mut self) -> FlowTrace {
        let loops = self.clone().find_loops();
        let funcs = self.clone().find_funcs(&loops);
        FlowTrace { funcs, loops }
    }

    fn next_block(block: &InstructionBlock) -> BlockId {
        block
            .last()
            .map(|lst| BlockId::from(lst.offset() + lst.size()))
            .unwrap()
    }

    fn find_funcs(&mut self, loops: &HashMap<BlockId, Loop>) -> HashMap<BlockId, Func> {
        let mut funcs: HashMap<BlockId, Func> = HashMap::new();

        for (id, block) in self.blocks {
            let last = if let Some(last) = block.last() {
                last
            } else {
                continue;
            };
            if last.1 != OpCode::Jump {
                continue;
            }

            if block.len() < 2 {
                continue;
            }

            let call_addr = if let Some(inst) = block.get(block.len() - 2) {
                if let OpCode::Push(vec) = &inst.1 {
                    let val = U256::from(vec.as_slice());
                    BlockId::from(val.as_usize())
                } else {
                    continue;
                }
            } else {
                continue;
            };

            let func = funcs.entry(call_addr).or_insert_with(|| Func {
                entry_point: call_addr,
                calls: Default::default(),
            });
            func.calls.insert(
                *id,
                Call {
                    entry_point: *id,
                    return_point: Self::next_block(block),
                },
            );
        }

        funcs
            .into_iter()
            .filter(|(id, fun)| self.check_func(id, fun, loops))
            .collect()
    }

    fn check_func(&self, id: &BlockId, fun: &Func, loops: &HashMap<BlockId, Loop>) -> bool {
        true
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
enum LoopCandidate {
    Candidate(BlockId, Vec<BlockId>),
}

#[derive(Debug)]
pub struct Func {
    pub entry_point: BlockId,
    pub calls: HashMap<BlockId, Call>,
}

#[derive(Debug)]
pub struct Call {
    pub entry_point: BlockId,
    pub return_point: BlockId,
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

#[derive(Debug)]
pub struct FlowTrace {
    pub funcs: HashMap<BlockId, Func>,
    pub loops: HashMap<BlockId, Loop>,
}
