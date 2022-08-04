use crate::bytecode::block::InstructionBlock;
use crate::bytecode::executor::types::U256;
use crate::{BlockId, OpCode};
use anyhow::Error;
use std::collections::{BTreeMap, VecDeque};
use std::fmt::Write;
use std::usize;

pub struct FlowBuilder<'a> {
    call_stack: Vec<BlockId>,
    block: BlockId,
    blocks: &'a BTreeMap<BlockId, InstructionBlock>,
}

impl<'a> FlowBuilder<'a> {
    pub fn new(blocks: &'a BTreeMap<BlockId, InstructionBlock>) -> FlowBuilder<'a> {
        FlowBuilder {
            call_stack: Vec::new(),
            block: Default::default(),
            blocks,
        }
    }

    pub fn make_flow(&mut self) -> Vec<Flow> {
        let mut cnd_branches = Vec::new();

        let mut branch_stack: Vec<BranchingState> = Vec::new();

        self.block = BlockId::default();
        'pc: loop {
            let next = self.exec_block();

            for branch in branch_stack.iter_mut() {
                branch.push_block(self.block);
            }

            match next {
                Next::Next(next) => {
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
                        .find(|b| b.jmp().block == self.block)
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
        Self::map_cnd_branches(cnd_branches)
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
                    OpCode::Jump => return Next::Next(ops.remove(0)),
                    OpCode::JumpIf => {
                        let jmp = ops.remove(0);
                        return Next::Cnd(jmp, BlockId(inst.next()));
                    }
                    OpCode::Return | OpCode::Stop | OpCode::Revert | OpCode::SelfDestruct => {
                        return Next::Stop
                    }
                    OpCode::Dup(_) => {
                        let new_item = ops[ops.len() - 1].clone();
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
                .map(|last| Next::Next(BlockId(last.next())))
                .unwrap_or(Next::Stop)
        } else {
            Next::Stop
        }
    }

    fn map_cnd_branches(elements: Vec<CndBranch>) -> Vec<Flow> {
        if elements.is_empty() {
            return vec![];
        }
        let mut blocks = elements
            .into_iter()
            .map(|flow| (flow.block(), flow))
            .collect::<BTreeMap<BlockId, CndBranch>>();

        let first_block = blocks.keys().next().unwrap().clone();
        let first_block = blocks.remove(&first_block).unwrap();

        let flow = Self::map_block(first_block, &mut blocks);

        let mut buffer = String::new();
        print_flows(&mut buffer, &flow, 0).unwrap();
        println!("{}", buffer);

        flow
    }

    fn map_block(element: CndBranch, blocks: &BTreeMap<BlockId, CndBranch>) -> Vec<Flow> {
        Self::map_if(element, blocks)
    }

    fn map_if_brunch(branch: &Branch, branch_map: &BTreeMap<BlockId, CndBranch>) -> Vec<Flow> {
        let blocks = &branch.blocks;
        if blocks.is_empty() {
            return vec![];
        }
        if branch.is_loop {
            vec![Flow::Loop(LoopFlow {
                loop_br: Self::map_branch(&blocks, branch_map),
            })]
        } else {
            Self::map_branch(&blocks, branch_map)
        }
    }

    fn map_if(mut if_block: CndBranch, branch_map: &BTreeMap<BlockId, CndBranch>) -> Vec<Flow> {
        let common_tail = if_block.take_common_fail().into_iter().collect::<Vec<_>>();

        let mut seq = vec![];
        seq.push(Flow::IF(IfFlow {
            jmp: if_block.jmp,
            true_br: Self::map_if_brunch(&if_block.true_br, branch_map),
            false_br: Self::map_if_brunch(&if_block.false_br, branch_map),
        }));

        if !common_tail.is_empty() {
            seq.extend(Self::map_branch(&common_tail, branch_map));
        }
        seq
    }

    fn map_branch(blocks: &[BlockId], elements: &BTreeMap<BlockId, CndBranch>) -> Vec<Flow> {
        let mut seq = Vec::new();
        if blocks.is_empty() {
            return seq;
        }

        let mut index = 0;
        loop {
            if blocks.len() <= index {
                break;
            }

            let block = blocks[index];
            if let Some(element) = elements.get(&block) {
                seq.extend(Self::map_block(element.clone(), elements));
                index += element.len();
            } else {
                index += 1;
                seq.push(Flow::Block(block));
            }
        }
        seq
    }
}

#[derive(Clone, Debug)]
pub enum BranchingState {
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
    block: BlockId,
    true_br: BlockId,
    false_br: BlockId,
}

#[derive(Debug, Clone)]
pub struct CndBranch {
    jmp: CndJmp,
    true_br: Branch,
    false_br: Branch,
}

#[derive(Debug, Clone, Default)]
pub struct Branch {
    pub end: BlockId,
    pub blocks: Vec<BlockId>,
    pub is_loop: bool,
}

impl Branch {
    pub fn set_end(&mut self, end: BlockId) {
        self.end = end;
    }

    pub fn set_loop(&mut self, is_loop: bool) {
        self.is_loop = is_loop;
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

        let (large, small) = if self.true_br.blocks.len() > self.false_br.blocks.len() {
            (&self.true_br.blocks, &self.false_br.blocks)
        } else {
            (&self.false_br.blocks, &self.true_br.blocks)
        };

        for (first_br, second_dr) in large.iter().rev().zip(small.iter().rev()) {
            if first_br == second_dr {
                tail.push_front(*first_br);
            } else {
                break;
            }
        }
        tail
    }
}

#[derive(Debug, Clone)]
pub struct LoopFlow {
    loop_br: Vec<Flow>,
}

#[derive(Clone, Copy, Debug)]
enum Next {
    Next(BlockId),
    Stop,
    Cnd(BlockId, BlockId),
}

impl CndBranch {
    pub fn block(&self) -> BlockId {
        self.jmp.block
    }

    pub fn len(&self) -> usize {
        self.true_br.blocks.len() + self.false_br.blocks.len()
    }
}

#[derive(Debug, Clone)]
pub enum Flow {
    Block(BlockId),
    Loop(LoopFlow),
    IF(IfFlow),
    Sequence(Vec<Flow>),
}

#[derive(Debug, Clone)]
pub struct IfFlow {
    pub jmp: CndJmp,
    pub true_br: Vec<Flow>,
    pub false_br: Vec<Flow>,
}

fn print_flows<W: Write>(buf: &mut W, vec: &[Flow], width: usize) -> Result<(), Error> {
    for flow in vec {
        print_flow(buf, flow, width)?;
    }
    Ok(())
}

fn print_flow<W: Write>(buf: &mut W, flow: &Flow, width: usize) -> Result<(), Error> {
    match flow {
        Flow::Block(seq) => {
            writeln!(buf, "{:width$}0x{}", " ", seq)?;
        }
        Flow::Loop(loop_) => {
            writeln!(buf, "{:width$}loop {{", " ")?;
            print_flows(buf, &loop_.loop_br, width + 4)?;
            writeln!(buf, "{:width$}}}", " ")?;
        }
        Flow::IF(if_) => {
            writeln!(buf, "{:width$}if ({}) {{", " ", if_.jmp.block)?;
            print_flows(buf, &if_.true_br, width + 4)?;
            writeln!(buf, "{:width$}}} else {{", " ")?;
            print_flows(buf, &if_.false_br, width + 4)?;
            writeln!(buf, "{:width$}}}", " ")?;
        }
        Flow::Sequence(flow) => {
            print_flows(buf, flow, width)?;
        }
    }
    Ok(())
}
