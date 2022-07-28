use crate::bytecode::block::InstructionBlock;
use crate::bytecode::executor::types::U256;
use crate::bytecode::instruction::Offset;
use crate::{BlockId, OpCode};
use std::collections::{BTreeMap, HashSet, VecDeque};
use std::usize;

pub struct Predictor<'a> {
    call_stack: Vec<BlockId>,
    block: BlockId,
    blocks: &'a BTreeMap<BlockId, InstructionBlock>,
}

impl<'a> Predictor<'a> {
    pub fn new(blocks: &'a BTreeMap<BlockId, InstructionBlock>) -> Predictor<'a> {
        Predictor {
            call_stack: Vec::new(),
            block: Default::default(),
            blocks,
        }
    }

    pub fn find_elements(&mut self) -> Elements {
        let mut flow = Elements::new();

        let mut branch_stack: Vec<BranchingState> = Vec::new();
        let mut branching_index = HashSet::new();

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
                        if let Some(if_el) = branching.make_if() {
                            flow.add_if(if_el);
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
                    branching_index.insert(true_br);
                    branching_index.insert(false_br);
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

        Self::canonize(flow)
    }

    fn canonize(elements: Elements) -> Elements {
        if elements.branches.is_empty() {
            return elements;
        }
        println!("blocks: {:?}", elements.branches);

        let mut blocks = elements
            .branches
            .into_iter()
            .map(|flow| (flow.block(), flow))
            .collect::<BTreeMap<BlockId, Element>>();

        let first_block = blocks.keys().next().unwrap().clone();
        let first_block = blocks.remove(&first_block).unwrap();

        match first_block {
            Element::IF(mut if_block) => {
                let flow = Self::canonize_if(if_block, &mut blocks);
                println!("flow: {:?}", flow);
            }
            Element::Loop(_) => {}
        }

        Elements { branches: vec![] }
    }

    fn canonize_element(element: Element, blocks: &BTreeMap<BlockId, Element>) -> Vec<Flow> {
        match element {
            Element::IF(if_) => Self::canonize_if(if_, blocks),
            Element::Loop(loop_) => {
                todo!()
            }
        }
    }

    fn canonize_if(mut if_block: IfElement, blocks: &BTreeMap<BlockId, Element>) -> Vec<Flow> {
        let common_tail = if_block.take_common_fail().into_iter().collect::<Vec<_>>();

        let true_br = if !if_block.true_br_blocks.is_empty() {
            Self::canonize_branch(&if_block.true_br_blocks, blocks)
        } else {
            vec![]
        };

        let false_br = if !if_block.false_br_blocks.is_empty() {
            Self::canonize_branch(&if_block.false_br_blocks, blocks)
        } else {
            vec![]
        };

        let mut seq = vec![];
        seq.push(Flow::IF(IfFlow {
            jmp: if_block.jmp,
            true_br,
            false_br,
        }));

        seq.extend(Self::canonize_branch(&common_tail, blocks));
        seq
    }

    fn canonize_branch(mut blocks: &[BlockId], elements: &BTreeMap<BlockId, Element>) -> Vec<Flow> {
        let mut seq = Vec::new();
        println!("block: {:?}", blocks);
        let index = 0;
        loop {
            let block = blocks[index];
            if let Some(element) = elements.get(&block) {
                seq.extend(Self::canonize_element(element.clone(), elements));
            } else {
                break;
            }
        }
        /*
        blocks: [
        IF(IfElement { jmp: CndJmp { block: 017c, true_br: 01b1, false_br: 01a9 }, pc_after_true: 0061, true_br_blocks: [01b1, 0099, 009c, 0054, 011c, 010d, 00aa, 0116, 0131, 0061], pc_after_false: 0137, false_br_blocks: [01a9, 0137] }),
        IF(IfElement { jmp: CndJmp { block: 017c, true_br: 01b1, false_br: 01a9 }, pc_after_true: 0061, true_br_blocks: [01b1, 0086, 009c, 0054, 011c, 010d, 00aa, 0116, 0131, 0061], pc_after_false: 0137, false_br_blocks: [01a9, 0137] }),
        IF(IfElement { jmp: CndJmp { block: 006a, true_br: 008d, false_br: 007b }, pc_after_true: 0137, true_br_blocks: [008d, 0166, 00aa, 0171, 00aa, 017c, 01b1, 0099, 009c, 0054, 011c, 010d, 00aa, 0116, 0131, 0061, 01a9, 0137], pc_after_false: 0137, false_br_blocks: [007b, 0166, 00aa, 0171, 00aa, 017c, 01b1, 0086, 009c, 0054, 011c, 010d, 00aa, 0116, 0131, 0061, 01a9, 0137] }),
        IF(IfElement { jmp: CndJmp { block: 00bd, true_br: 00c8, false_br: 00c4 }, pc_after_true: 0137, true_br_blocks: [00c8, 00da, 0104, 004f, 006a, 008d, 0166, 00aa, 0171, 00aa, 017c, 01b1, 0099, 009c, 0054, 011c, 010d, 00aa, 0116, 0131, 0061, 01a9, 0137, 007b, 0166, 00aa, 0171, 00aa, 017c, 01b1, 0086, 009c, 0054, 011c, 010d, 00aa, 0116, 0131, 0061, 01a9, 0137], pc_after_false: 00c4, false_br_blocks: [00c4] }),
        IF(IfElement { jmp: CndJmp { block: 00e0, true_br: 00f6, false_br: 00ee }, pc_after_true: 00c4, true_br_blocks: [00f6, 00cb, 00b4, 00aa, 00bd, 00c8, 00da, 0104, 004f, 006a, 008d, 0166, 00aa, 0171, 00aa, 017c, 01b1, 0099, 009c, 0054, 011c, 010d, 00aa, 0116, 0131, 0061, 01a9, 0137, 007b, 0166, 00aa, 0171, 00aa, 017c, 01b1, 0086, 009c, 0054, 011c, 010d, 00aa, 0116, 0131, 0061, 01a9, 0137, 00c4], pc_after_false: 00a5, false_br_blocks: [00ee, 00a5] }),
        IF(IfElement { jmp: CndJmp { block: 0024, true_br: 003a, false_br: 0035 }, pc_after_true: 00a5, true_br_blocks: [003a, 00e0, 00f6, 00cb, 00b4, 00aa, 00bd, 00c8, 00da, 0104, 004f, 006a, 008d, 0166, 00aa, 0171, 00aa, 017c, 01b1, 0099, 009c, 0054, 011c, 010d, 00aa, 0116, 0131, 0061, 01a9, 0137, 007b, 0166, 00aa, 0171, 00aa, 017c, 01b1, 0086, 009c, 0054, 011c, 010d, 00aa, 0116, 0131, 0061, 01a9, 0137, 00c4, 00ee, 00a5], pc_after_false: 0035, false_br_blocks: [0035] }),
        IF(IfElement { jmp: CndJmp { block: 0000, true_br: 0035, false_br: 0024 }, pc_after_true: 0035, true_br_blocks: [0035], pc_after_false: 0035, false_br_blocks: [0024, 003a, 00e0, 00f6, 00cb, 00b4, 00aa, 00bd, 00c8, 00da, 0104, 004f, 006a, 008d, 0166, 00aa, 0171, 00aa, 017c, 01b1, 0099, 009c, 0054, 011c, 010d, 00aa, 0116, 0131, 0061, 01a9, 0137, 007b, 0166, 00aa, 0171, 00aa, 017c, 01b1, 0086, 009c, 0054, 011c, 010d, 00aa, 0116, 0131, 0061, 01a9, 0137, 00c4, 00ee, 00a5, 0035] })]
         */

        seq
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
                println!("inst:{} stack {:?}", &inst, self.call_stack);
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
}

#[derive(Clone, Debug)]
pub enum BranchingState {
    JumpIf {
        jmp: CndJmp,
        stack: Vec<BlockId>,
        true_br_blocks: Vec<BlockId>,
    },
    TrueBranch {
        jmp: CndJmp,
        pc_after_true: BlockId,
        true_br_blocks: Vec<BlockId>,
        stack: Vec<BlockId>,
        false_br_blocks: Vec<BlockId>,
    },
    IF {
        jmp: CndJmp,
        pc_after_true: BlockId,
        true_br_blocks: Vec<BlockId>,
        pc_after_false: BlockId,
        stack: Vec<BlockId>,
        false_br_blocks: Vec<BlockId>,
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
            true_br_blocks: Vec::new(),
        }
    }

    pub fn set_end(self, end: BlockId) -> BranchingState {
        match self {
            BranchingState::JumpIf {
                jmp,
                stack,
                true_br_blocks: blocks,
            } => BranchingState::TrueBranch {
                jmp,
                pc_after_true: end,
                true_br_blocks: blocks,
                stack,
                false_br_blocks: Vec::new(),
            },
            BranchingState::TrueBranch {
                jmp,
                pc_after_true,
                true_br_blocks,
                stack,
                false_br_blocks,
            } => BranchingState::IF {
                jmp,
                pc_after_true,
                true_br_blocks,
                pc_after_false: end,
                stack,
                false_br_blocks,
            },
            BranchingState::IF { .. } => self,
        }
    }

    pub fn make_if(&self) -> Option<IfElement> {
        match self.clone() {
            BranchingState::IF {
                jmp,
                pc_after_true,
                true_br_blocks,
                pc_after_false,
                stack: _,
                false_br_blocks,
            } => Some(IfElement {
                jmp,
                pc_after_true,
                true_br_blocks,
                pc_after_false,
                false_br_blocks,
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
            BranchingState::JumpIf {
                true_br_blocks: blocks,
                ..
            } => {
                blocks.push(block_id);
            }
            BranchingState::TrueBranch {
                false_br_blocks, ..
            } => {
                false_br_blocks.push(block_id);
            }
            BranchingState::IF { .. } => {}
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
pub struct IfElement {
    jmp: CndJmp,
    pc_after_true: BlockId,
    true_br_blocks: Vec<BlockId>,
    pc_after_false: BlockId,
    false_br_blocks: Vec<BlockId>,
}

impl IfElement {
    pub fn take_common_fail(&mut self) -> VecDeque<BlockId> {
        let mut tail = VecDeque::new();

        loop {
            let last_true_br = self.true_br_blocks.last();
            let last_false_br = self.false_br_blocks.last();

            if let (Some(true_br), Some(false_br)) = (last_true_br, last_false_br) {
                if *true_br == *false_br {
                    tail.push_front(*true_br);
                    self.true_br_blocks.pop();
                    self.false_br_blocks.pop();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let (large, small) = if self.true_br_blocks.len() > self.false_br_blocks.len() {
            (&self.true_br_blocks, &self.false_br_blocks)
        } else {
            (&self.false_br_blocks, &self.true_br_blocks)
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
pub struct Loop {
    jmp: CndJmp,
}

#[derive(Clone, Copy, Debug)]
enum Next {
    Next(BlockId),
    Stop,
    Cnd(BlockId, BlockId),
}

#[derive(Debug, Clone)]
pub enum Element {
    IF(IfElement),
    Loop(Loop),
}

impl Element {
    pub fn block(&self) -> BlockId {
        match self {
            Element::IF(if_) => if_.jmp.block,
            Element::Loop(loop_) => loop_.jmp.block,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Elements {
    pub branches: Vec<Element>,
}

impl Elements {
    pub fn new() -> Elements {
        Elements { branches: vec![] }
    }

    pub fn add_if(&mut self, branching: IfElement) {
        self.branches.push(Element::IF(branching));
    }
}

#[derive(Debug, Clone)]
pub enum Flow {
    Sec(BlockId),
    Loop(Loop),
    IF(IfFlow),
}

#[derive(Debug, Clone)]
pub struct IfFlow {
    pub jmp: CndJmp,
    pub true_br: Vec<Flow>,
    pub false_br: Vec<Flow>,
}
