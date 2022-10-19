use std::collections::{BTreeMap, HashMap, HashSet};
use std::mem;

use anyhow::{anyhow, Error};

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
        let loops = self.clone().find_loops()?;
        let funcs = self.clone().find_funcs(&loops);
        Ok(FlowTrace { funcs, loops })
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
                input: vec![],
                output: vec![],
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
            .filter_map(|(offset, mut fun)| {
                for call in &fun.calls {
                    match self.calc_func_io(call.1, loops) {
                        Ok((input, output)) => {
                            if fun.input.is_empty() && fun.output.is_empty() {
                                fun.input = input;
                                fun.output = output;
                            } else {
                                if fun.input != input || fun.output != output {
                                    return None;
                                }
                            }
                        }
                        Err(err) => {
                            log::warn!("Failed to calculate function IO: {}", err);
                            return None;
                        }
                    }
                }

                Some((offset, fun))
            })
            .collect()
    }

    fn calc_func_io(
        &self,
        call: &Call,
        loops: &HashMap<Offset, Loop>,
    ) -> Result<(Vec<StackItem>, Vec<StackItem>), Error> {
        let mut block_id = call.entry_point;
        let ret = call.return_point;

        let mut br_stack = Vec::new();

        let mut io_states = Vec::new();
        let mut visited_loops = HashSet::new();
        let mut entry_exec = Executor::default();
        let mut fun_exec = Executor::default();

        loop {
            let block = self.blocks.get(&block_id).ok_or_else(|| {
                anyhow!(
                    "Block with id {} not found. Blocks: {:?}",
                    block_id,
                    self.blocks
                )
            })?;
            let next = entry_exec.exec(block);

            if call.entry_point != block_id {
                fun_exec.exec(block);
            }

            match next {
                Next::Jmp(val) => {
                    let jmp = val.as_positive()?;
                    if jmp == ret {
                        io_states.push(fun_exec);
                        return self.branch_io_to_fun_io(io_states);
                    }
                    if loops.contains_key(&jmp) {
                        if !visited_loops.insert(jmp) {
                            if let Some((br, entry_br_exec, mut func_br_exec)) = br_stack.pop() {
                                block_id = br;
                                entry_exec = entry_br_exec;
                                mem::swap(&mut fun_exec, &mut func_br_exec);
                                io_states.push(func_br_exec);
                                continue;
                            } else {
                                return Err(anyhow!("Not a function:{}", call.entry_point));
                            }
                        }
                    }
                    block_id = jmp;
                }
                Next::Stop => {
                    if let Some((br, entry_br_exec, mut func_br_exec)) = br_stack.pop() {
                        block_id = br;
                        entry_exec = entry_br_exec;
                        mem::swap(&mut fun_exec, &mut func_br_exec);
                        io_states.push(func_br_exec);
                    } else {
                        return Err(anyhow!("Not a function:{}", call.entry_point));
                    }
                }
                Next::Cnd(true_br, false_br) => {
                    let true_br = true_br.as_positive()?;
                    let false_br = false_br.as_positive()?;
                    br_stack.push((false_br, entry_exec.clone(), fun_exec.clone()));
                    block_id = true_br;
                }
            }
        }
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

    fn branch_io_to_fun_io(
        &self,
        mut executors: Vec<Executor>,
    ) -> Result<(Vec<StackItem>, Vec<StackItem>), Error> {
        let exec = executors
            .pop()
            .ok_or_else(|| anyhow!("Empty function io"))?;
        let (mut input, mut output) = exec.into_io();

        for exec in executors {
            let (i, o) = exec.into_io();
            if i.len() > input.len() {
                input = i;
            }
            if o.len() > output.len() {
                output = o;
            }
        }

        return Ok((input, output));
    }
}

#[derive(Debug, Clone, Default)]
pub struct LoopCtx {
    pub block: Offset,
    pub output: Vec<StackItem>,
    pub input: Vec<StackItem>,
}

impl LoopCtx {
    pub fn new(block: Offset, exec: &Executor) -> Self {
        LoopCtx {
            block,
            output: exec.call_stack().to_vec(),
            input: exec.negative_stack().to_vec(),
        }
    }
}

#[derive(Debug)]
pub struct Func {
    pub entry_point: Offset,
    pub calls: HashMap<Offset, Call>,
    pub input: Vec<StackItem>,
    pub output: Vec<StackItem>,
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
    pub funcs: HashMap<Offset, Func>,
    pub loops: HashMap<Offset, Loop>,
}

pub type ID = usize;

#[derive(Debug, Clone)]
pub struct BlockIO {
    pub inputs: Vec<(ID, Offset)>,
    pub outputs: BTreeMap<Offset, StackItem>,
}
