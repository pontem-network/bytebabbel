use std::collections::{BTreeMap, HashMap, HashSet};

use anyhow::{anyhow, Context as ErrContext, Error};

use crate::bytecode::block::InstructionBlock;
use crate::bytecode::tracing::exec::{Executor, Next, StackItem};
use crate::{Offset, OpCode, U256};

#[derive(Clone, Debug)]
pub struct Tracer<'a> {
    blocks: &'a HashMap<Offset, InstructionBlock>,
    executor: Executor,
}

impl<'a> Tracer<'a> {
    pub fn new(blocks: &'a HashMap<Offset, InstructionBlock>) -> Self {
        Self {
            blocks,
            executor: Executor::default(),
        }
    }

    pub fn trace(&mut self) -> Result<FlowTrace, Error> {
        let io = self.calculate_io()?;
        let loops = self.clone().find_loops()?;
        let funcs = self.clone().find_funcs(&loops);
        Ok(FlowTrace { io, funcs, loops })
    }

    fn next_block(block: &InstructionBlock) -> Offset {
        block.last().map(|lst| lst.offset() + lst.size()).unwrap()
    }

    fn find_funcs(&mut self, loops: &HashMap<Offset, Loop>) -> HashMap<Offset, Func> {
        let mut funcs: HashMap<Offset, Func> = HashMap::new();

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
                    Offset::from(val)
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
            .filter(|(_, fun)| fun.calls.len() > 1)
            .filter(|(_, fun)| !loops.contains_key(&fun.entry_point))
            .filter(|(_, fun)| {
                fun.calls
                    .iter()
                    .all(|(_, call)| self.is_function(fun, call, loops))
            })
            .collect()
    }

    fn is_function(&self, fun: &Func, call: &Call, loops: &HashMap<Offset, Loop>) -> bool {
        let mut block = call.entry_point;

        let mut exec = Executor::default();
        loop {
            let block = self
                .blocks
                .get(&block)
                .ok_or_else(|| {
                    format!(
                        "Block with id {} not found. Blocks: {:?}",
                        block, self.blocks
                    )
                })
                .unwrap();
        }

        true
    }

    fn find_loops(&mut self) -> Result<HashMap<Offset, Loop>, Error> {
        let mut id = Offset::default();
        let mut stack: Vec<Fork> = vec![];
        let mut loop_candidates: HashMap<Offset, (Offset, Vec<Offset>)> = HashMap::new();
        let mut loops: HashMap<Offset, Loop> = HashMap::new();
        let mut breaks: HashMap<Offset, Offset> = HashMap::new();
        loop {
            let block = self
                .blocks
                .get(&id)
                .ok_or_else(|| format!("Block with id {} not found. Blocks: {:?}", id, self.blocks))
                .unwrap();

            let res = self.executor.exec(block);
            match res {
                Next::Jmp(jmp) => {
                    let jmp = jmp.as_positive()?;
                    if let Some(lp) = breaks.get(&jmp) {
                        loops.get_mut(lp).unwrap().breaks.insert(id);
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
                                            continuous: *lp.last().unwrap(),
                                            breaks: HashSet::new(),
                                            fork: fork.clone(),
                                            loop_ctx: Default::default(),
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
                        return Ok(loops);
                    }
                },
                Next::Cnd(true_br, false_br) => {
                    let true_br = true_br.as_positive()?;
                    let false_br = false_br.as_positive()?;
                    if let Some(lp) = breaks.get(&true_br) {
                        loops.get_mut(lp).unwrap().breaks.insert(id);
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
                        loops.get_mut(lp).unwrap().breaks.insert(id);
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

    fn calculate_io(&self) -> Result<HashMap<Offset, BlockIO>, Error> {
        let mut io: HashMap<Offset, BlockIO> = HashMap::new();
        for (id, block) in self.blocks {
            let mut exec = Executor::default();
            let res = exec.exec_one(block);

            let outputs = res.output.into_iter().map(|i| (i.offset(), i)).collect();

            let inputs = res
                .input
                .into_iter()
                .map(|i| {
                    i.as_negative()
                        .ok_or_else(|| anyhow!("Invalid input: {:?}. Block:{}", i, id))
                })
                .collect::<Result<_, _>>()?;

            io.insert(*id, BlockIO { inputs, outputs });
        }
        Ok(io)
    }
}

#[derive(Debug, Clone, Default)]
pub struct LoopCtx {
    pub block: Offset,
    pub output: Vec<StackItem>,
    pub input: HashSet<StackItem>,
}

impl LoopCtx {
    pub fn new(block: Offset, exec: &Executor) -> Self {
        LoopCtx {
            block,
            output: exec.call_stack().clone(),
            input: exec.negative_item_used().clone(),
        }
    }
}

pub struct Context {
    pub executor: Executor,
    pub false_br: Offset,
}

#[derive(Debug)]
pub struct Func {
    pub entry_point: Offset,
    pub calls: HashMap<Offset, Call>,
}

#[derive(Debug)]
pub struct Call {
    pub entry_point: Offset,
    pub return_point: Offset,
}

#[derive(Clone, Debug)]
pub struct Fork {
    pub id: Offset,
    pub exec: Executor,
    pub state: (Offset, Offset),
    pub next_br: Option<Offset>,
}

#[derive(Debug, Clone)]
pub struct Loop {
    pub root: Offset,
    pub loop_exit: Offset,
    pub loop_br: Offset,
    pub continuous: Offset,
    pub breaks: HashSet<Offset>,
    pub fork: Fork,
    pub loop_ctx: LoopCtx,
}

#[derive(Debug)]
pub struct FlowTrace {
    pub io: HashMap<Offset, BlockIO>,
    pub funcs: HashMap<Offset, Func>,
    pub loops: HashMap<Offset, Loop>,
}

pub type ID = usize;

#[derive(Debug, Clone)]
pub struct BlockIO {
    pub inputs: Vec<(ID, Offset)>,
    pub outputs: BTreeMap<Offset, StackItem>,
}
