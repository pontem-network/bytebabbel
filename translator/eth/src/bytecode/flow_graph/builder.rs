#![allow(dead_code)]
use crate::bytecode::block::InstructionBlock;
use crate::bytecode::flow_graph::flow::Flow;
use crate::bytecode::flow_graph::mapper::map_flow;
use crate::bytecode::tracing::tracer::{FlowTrace, Tracer};
use crate::{BlockId, OpCode};
use anyhow::Error;
use primitive_types::U256;
use std::collections::{HashMap, VecDeque};
use std::usize;

pub struct FlowBuilder<'a> {
    call_stack: Vec<BlockId>,
    block: BlockId,
    blocks: &'a HashMap<BlockId, InstructionBlock>,
    flow_trace: FlowTrace,
}

impl<'a> FlowBuilder<'a> {
    pub fn new(blocks: &'a HashMap<BlockId, InstructionBlock>) -> Result<FlowBuilder<'a>, Error> {
        Ok(FlowBuilder {
            call_stack: Vec::new(),
            block: Default::default(),
            blocks,
            flow_trace: Tracer::new(blocks).trace()?,
        })
    }

    pub fn make_flow(&mut self) -> Flow {
        let mut cnd_branches = Vec::new();

        let mut branch_stack: Vec<BranchingState> = Vec::new();
        let mut root = Vec::<BlockId>::new();

        self.block = BlockId::default();
        'pc: loop {
            let next = self.exec_block();

            for branch in branch_stack.iter_mut() {
                branch.push_block(self.block);
            }

            if branch_stack.is_empty() && !cnd_branches.is_empty() {
                root.push(self.block);
            }

            match next {
                Next::Jmp(next) => {
                    self.block = next;
                }
                Next::Stop => 'branch: loop {
                    if let Some(branching) = branch_stack.pop() {
                        let branching = branching.set_end(self.block);
                        self.call_stack = branching.stack().clone();
                        if let Some(if_el) = branching.complete() {
                            cnd_branches.push(if_el);
                            continue;
                        } else {
                            self.block = branching.false_br();
                            branch_stack.push(branching);
                            break 'branch;
                        }
                    } else {
                        break 'pc;
                    }
                },
                Next::Cnd(true_br, false_br) => {
                    let loop_br = branch_stack
                        .iter_mut()
                        .filter(|b| b.jmp().block == self.block)
                        .find(|b| self.flow_trace.loops.contains_key(&b.jmp().block))
                        .map(|b| {
                            b.set_loop();
                            true
                        })
                        .unwrap_or_default();

                    if loop_br {
                        for branch in branch_stack.iter_mut() {
                            branch.pop_block();
                        }

                        'branch: loop {
                            if let Some(branching) = branch_stack.pop() {
                                let branching = branching.set_end(self.block);
                                self.call_stack = branching.stack().clone();
                                if let Some(if_el) = branching.complete() {
                                    cnd_branches.push(if_el);
                                    continue;
                                } else {
                                    self.block = branching.false_br();
                                    branch_stack.push(branching);
                                    break 'branch;
                                }
                            } else {
                                break 'pc;
                            }
                        }
                    } else {
                        branch_stack.push(BranchingState::new(
                            self.block,
                            true_br,
                            false_br,
                            self.call_stack.clone(),
                        ));
                        self.block = true_br;
                    }
                }
            }
        }

        if root.is_empty() {
            map_flow(cnd_branches)
        } else {
            Flow::Sequence(
                root.into_iter()
                    .map(Flow::Block)
                    .chain([map_flow(cnd_branches)])
                    .collect(),
            )
        }
    }

    fn pop_stack(&mut self, count: usize) -> Vec<BlockId> {
        let mut res = Vec::with_capacity(count);
        for _ in 0..count {
            res.push(self.call_stack.pop().unwrap_or_default());
        }
        res
    }

    fn push_stack(&mut self, to_push: Vec<BlockId>) {
        self.call_stack.extend(to_push.into_iter().rev());
    }

    fn exec_block(&mut self) -> Next {
        if let Some(block) = self.blocks.get(&self.block) {
            for inst in block.iter() {
                let mut ops = self.pop_stack(inst.pops());
                match &inst.1 {
                    OpCode::Jump => return Next::Jmp(ops.remove(0)),
                    OpCode::JumpIf => {
                        let jmp = ops.remove(0);
                        return Next::Cnd(jmp, BlockId(inst.next()));
                    }
                    OpCode::Return | OpCode::Stop | OpCode::Revert | OpCode::SelfDestruct => {
                        return Next::Stop
                    }
                    OpCode::Dup(_) => {
                        let new_item = ops[ops.len() - 1];
                        ops.insert(0, new_item);
                        self.push_stack(ops);
                    }
                    OpCode::Swap(_) => {
                        let last_index = ops.len() - 1;
                        ops.swap(0, last_index);
                        self.push_stack(ops);
                    }
                    OpCode::Push(val) => {
                        let val = U256::from(val.as_slice());
                        if val <= U256::from(u32::MAX) {
                            self.push_stack(vec![BlockId::from(val.as_usize())]);
                        } else {
                            self.push_stack(vec![BlockId::default()]);
                        }
                    }
                    _ => {
                        let pushes = inst.pushes();
                        if pushes > 0 {
                            self.push_stack((0..pushes).map(|_| BlockId::default()).collect());
                        }
                    }
                }
            }
            block
                .last()
                .map(|last| Next::Jmp(BlockId(last.next())))
                .unwrap_or(Next::Stop)
        } else {
            Next::Stop
        }
    }
}

#[derive(Clone, Debug)]
enum BranchingState {
    JumpIf {
        jmp: CndJmp,
        stack: Vec<BlockId>,
        true_br: Branch,
    },
    TrueBranch {
        jmp: CndJmp,
        true_br: Branch,
        false_br: Branch,
        stack: Vec<BlockId>,
    },
    IF {
        jmp: CndJmp,
        true_br: Branch,
        false_br: Branch,
        stack: Vec<BlockId>,
    },
}

impl BranchingState {
    pub fn new(block: BlockId, true_br: BlockId, false_br: BlockId, stack: Vec<BlockId>) -> Self {
        BranchingState::JumpIf {
            jmp: CndJmp {
                block,
                true_br,
                false_br,
            },
            stack,
            true_br: Branch::default(),
        }
    }

    pub fn set_loop(&mut self) {
        match self {
            BranchingState::JumpIf { true_br, .. } => {
                true_br.set_loop(true);
            }
            BranchingState::TrueBranch { false_br, .. } => {
                false_br.set_loop(true);
            }
            BranchingState::IF { .. } => {}
        }
    }

    pub fn set_end(self, end: BlockId) -> BranchingState {
        match self {
            BranchingState::JumpIf {
                jmp,
                stack,
                mut true_br,
            } => {
                true_br.set_end(end);
                BranchingState::TrueBranch {
                    jmp,
                    true_br,
                    false_br: Default::default(),
                    stack,
                }
            }
            BranchingState::TrueBranch {
                jmp,
                true_br,
                mut false_br,
                stack,
            } => {
                false_br.set_end(end);
                BranchingState::IF {
                    jmp,
                    true_br,
                    false_br,
                    stack,
                }
            }
            BranchingState::IF { .. } => self,
        }
    }

    pub fn complete(&self) -> Option<CndBranch> {
        match self.clone() {
            BranchingState::IF {
                jmp,
                true_br,
                false_br,
                stack: _,
            } => Some(CndBranch {
                jmp,
                true_br,
                false_br,
            }),
            _ => None,
        }
    }

    pub fn stack(&self) -> &Vec<BlockId> {
        match self {
            BranchingState::JumpIf { stack, .. } => stack,
            BranchingState::TrueBranch { stack, .. } => stack,
            BranchingState::IF { stack, .. } => stack,
        }
    }

    pub fn false_br(&self) -> BlockId {
        match self {
            BranchingState::JumpIf { jmp, .. } => jmp.false_br,
            BranchingState::TrueBranch { jmp, .. } => jmp.false_br,
            BranchingState::IF { jmp, .. } => jmp.false_br,
        }
    }

    pub fn push_block(&mut self, block_id: BlockId) {
        match self {
            BranchingState::JumpIf { true_br, .. } => {
                true_br.blocks.push(block_id);
            }
            BranchingState::TrueBranch { false_br, .. } => {
                false_br.blocks.push(block_id);
            }
            BranchingState::IF { .. } => {}
        }
    }

    pub fn pop_block(&mut self) {
        match self {
            BranchingState::JumpIf { true_br, .. } => {
                true_br.blocks.pop();
            }
            BranchingState::TrueBranch { false_br, .. } => {
                false_br.blocks.pop();
            }
            BranchingState::IF { .. } => {}
        }
    }

    pub fn jmp(&self) -> &CndJmp {
        match self {
            BranchingState::JumpIf { jmp, .. } => jmp,
            BranchingState::TrueBranch { jmp, .. } => jmp,
            BranchingState::IF { jmp, .. } => jmp,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CndJmp {
    pub block: BlockId,
    pub true_br: BlockId,
    pub false_br: BlockId,
}

#[derive(Debug, Clone)]
pub struct CndBranch {
    pub jmp: CndJmp,
    pub true_br: Branch,
    pub false_br: Branch,
}

#[derive(Debug, Clone, Default)]
pub struct Branch {
    pub end: BlockId,
    pub blocks: Vec<BlockId>,
    pub continue_blocks: Option<Continue>,
    pub is_loop: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Continue {
    pub loop_head: BlockId,
    pub continue_block: BlockId,
}

impl Branch {
    pub fn set_end(&mut self, end: BlockId) {
        self.end = end;
    }

    pub fn set_loop(&mut self, is_loop: bool) {
        self.is_loop = is_loop;
        let len = self.blocks.len();
        let loop_head = self.blocks[len - 1];
        let continue_block = self.blocks[len - 2];
        self.continue_blocks = Some(Continue {
            loop_head,
            continue_block,
        });
    }
}

impl CndBranch {
    pub fn take_common_fail(&mut self) -> VecDeque<BlockId> {
        let mut tail = VecDeque::new();

        loop {
            let last_true_br = self.true_br.blocks.last();
            let last_false_br = self.false_br.blocks.last();

            if let (Some(true_br), Some(false_br)) = (last_true_br, last_false_br) {
                if *true_br == *false_br {
                    tail.push_front(*true_br);
                    self.true_br.blocks.pop();
                    self.false_br.blocks.pop();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        tail
    }

    pub fn is_subset(&self, other: &[BlockId]) -> bool {
        let mut idx = 0;
        if self.block() != other[idx] {
            return false;
        }
        idx += 1;

        fn is_subset_branch(br: &Branch, other: &[BlockId]) -> bool {
            for (idx, block) in br.blocks.iter().enumerate() {
                if *block != other[idx] {
                    return false;
                }
            }
            true
        }

        if is_subset_branch(&self.true_br, &other[idx..]) {
            idx += self.true_br.blocks.len();
        } else {
            return false;
        }

        is_subset_branch(&self.false_br, &other[idx..])
    }
}

#[derive(Clone, Copy, Debug)]
enum Next {
    Jmp(BlockId),
    Stop,
    Cnd(BlockId, BlockId),
}

impl CndBranch {
    pub fn block(&self) -> BlockId {
        self.jmp.block
    }

    pub fn len(&self) -> usize {
        self.true_br.blocks.len() + self.false_br.blocks.len() + 1
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
