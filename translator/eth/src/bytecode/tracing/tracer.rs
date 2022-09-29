use crate::bytecode::block::InstructionBlock;
use crate::bytecode::instruction::Offset;
use crate::bytecode::tracing::exec::{Executor, Next, StackItem};
use crate::{BlockId, OpCode, U256};
use anyhow::{anyhow, Context as ErrContext, Error};
use std::collections::{BTreeMap, HashMap, HashSet};

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

    pub fn trace(&mut self) -> Result<FlowTrace, Error> {
        let io = self.calculate_io()?;
        let loops = self.clone().find_loops()?;
        let funcs = self.clone().find_funcs(&loops);
        Ok(FlowTrace { io, funcs, loops })
    }

    fn next_block(block: &InstructionBlock) -> BlockId {
        block
            .last()
            .map(|lst| BlockId::from(lst.offset() + lst.size() as Offset))
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
                    BlockId::from(val)
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

    fn check_func(&self, _id: &BlockId, _fun: &Func, _loops: &HashMap<BlockId, Loop>) -> bool {
        //todo filter out functions that are not really functions)
        true
    }

    fn find_loops(&mut self) -> Result<HashMap<BlockId, Loop>, Error> {
        let mut id = BlockId::default();
        let mut stack: Vec<Fork> = vec![];
        let mut loop_candidates: HashMap<BlockId, (BlockId, Vec<BlockId>)> = HashMap::new();
        let mut loops: HashMap<BlockId, Loop> = HashMap::new();
        let mut breaks: HashMap<BlockId, BlockId> = HashMap::new();
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

    fn calculate_io(&self) -> Result<HashMap<BlockId, BlockIO>, Error> {
        let mut io: HashMap<BlockId, BlockIO> = HashMap::new();
        for (id, block) in self.blocks {
            let mut exec = Executor::default();
            let res = exec.exec_one(block);

            let outputs = res
                .output
                .into_iter()
                .map(|i| (i.offset().0 as u64, i))
                .collect();

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

    pub fn fill_io(&self, lp: &mut Loop, loops: &HashMap<BlockId, Loop>) -> Result<(), Error> {
        let mut exec = Executor::default();
        let mut block_id = lp.root;
        let exit = lp.loop_exit;

        let mut ctx: Vec<Context> = vec![];
        loop {
            let block = self.blocks.get(&block_id).unwrap();
            let next = exec.exec(block);
            match next {
                Next::Jmp(jmp) => {
                    let jmp = jmp.as_positive().context("Invalid jmp")?;
                    if lp.root == jmp {
                        lp.loop_ctx = LoopCtx::new(block_id, &exec);
                        break;
                    }

                    if let Some(_lp) = loops.get(&jmp) {
                        todo!("Loop inside loop");
                    } else {
                        block_id = jmp;
                    }
                }
                Next::Stop => {
                    if let Some(ctx) = ctx.pop() {
                        exec = ctx.executor;
                        block_id = ctx.false_br;
                    }
                }
                Next::Cnd(true_br, false_br) => {
                    let true_br = true_br.as_positive().context("Invalid true branch")?;
                    let false_br = false_br.as_positive().context("Invalid false branch")?;

                    if true_br == exit {
                        block_id = false_br;
                        continue;
                    }

                    ctx.push(Context {
                        executor: exec.clone(),
                        false_br,
                    });
                    block_id = true_br;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct LoopCtx {
    pub block: BlockId,
    pub output: Vec<StackItem>,
    pub input: HashSet<StackItem>,
}

impl LoopCtx {
    pub fn new(block: BlockId, exec: &Executor) -> Self {
        LoopCtx {
            block,
            output: exec.call_stack().clone(),
            input: exec.negative_item_used().clone(),
        }
    }
}

pub struct Context {
    pub executor: Executor,
    pub false_br: BlockId,
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

#[derive(Debug, Clone)]
pub struct Loop {
    pub root: BlockId,
    pub loop_exit: BlockId,
    pub loop_br: BlockId,
    pub continuous: BlockId,
    pub breaks: HashSet<BlockId>,
    pub fork: Fork,
    pub loop_ctx: LoopCtx,
}

#[derive(Debug)]
pub struct FlowTrace {
    pub io: HashMap<BlockId, BlockIO>,
    pub funcs: HashMap<BlockId, Func>,
    pub loops: HashMap<BlockId, Loop>,
}

pub type ID = usize;

#[derive(Debug, Clone)]
pub struct BlockIO {
    pub inputs: Vec<(ID, BlockId)>,
    pub outputs: BTreeMap<Offset, StackItem>,
}
