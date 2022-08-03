use crate::bytecode::block::InstructionBlock;
use crate::bytecode::executor::types::U256;
use crate::{BlockId, OpCode};
use std::collections::{BTreeMap, VecDeque};
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

    pub fn find_elements(&mut self) -> Vec<Flow> {
        let mut cnd_branches = Vec::new();

        let mut branch_stack: Vec<BranchingState> = Vec::new();

        self.block = BlockId::default();
        'pc: loop {
            println!("Block: {}", self.block);
            let next = self.exec_block();

            for branch in branch_stack.iter_mut() {
                branch.push_block(self.block);
            }

            match next {
                Next::Next(next) => {
                    self.block = next;
                }
                Next::Stop => 'branch: loop {
                    println!("Stop:{}", self.block);
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
                        println!("Loop detected at {}", self.block);
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
        Self::canonize(cnd_branches)
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

    fn canonize(elements: Vec<CndBranch>) -> Vec<Flow> {
        if elements.is_empty() {
            return vec![];
        }
        println!("blocks: {:?}", elements);

        let mut blocks = elements
            .into_iter()
            .map(|flow| (flow.block(), flow))
            .collect::<BTreeMap<BlockId, CndBranch>>();

        let first_block = blocks.keys().next().unwrap().clone();
        let first_block = blocks.remove(&first_block).unwrap();

        let flow = Self::canonize_block(first_block, &mut blocks);
        println!("flow: {:?}", flow);
        flow
    }

    fn canonize_block(element: CndBranch, blocks: &BTreeMap<BlockId, CndBranch>) -> Vec<Flow> {
        Self::canonize_if(element, blocks)
    }

    fn canonize_if(mut if_block: CndBranch, blocks: &BTreeMap<BlockId, CndBranch>) -> Vec<Flow> {
        let common_tail = if_block.take_common_fail().into_iter().collect::<Vec<_>>();

        let true_br = if !if_block.true_br.blocks.is_empty() {
            Self::canonize_branch(&if_block.true_br.blocks, blocks)
        } else {
            vec![]
        };

        let false_br = if !if_block.false_br.blocks.is_empty() {
            Self::canonize_branch(&if_block.false_br.blocks, blocks)
        } else {
            vec![]
        };

        let mut seq = vec![];
        seq.push(Flow::IF(IfFlow {
            jmp: if_block.jmp,
            true_br,
            false_br,
        }));

        if !common_tail.is_empty() {
            seq.extend(Self::canonize_branch(&common_tail, blocks));
        }
        seq
    }

    fn canonize_branch(
        mut blocks: &[BlockId],
        elements: &BTreeMap<BlockId, CndBranch>,
    ) -> Vec<Flow> {
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
                seq.extend(Self::canonize_block(element.clone(), elements));
                index += element.len();
            } else {
                index += 1;
                seq.push(Flow::Sec(block));
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
        true_br: Brunch,
    },
    TrueBranch {
        jmp: CndJmp,
        true_br: Brunch,
        false_br: Brunch,
        stack: Vec<BlockId>,
    },
    IF {
        jmp: CndJmp,
        true_br: Brunch,
        false_br: Brunch,
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
            true_br: Brunch::default(),
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
    true_br: Brunch,
    false_br: Brunch,
}

#[derive(Debug, Clone, Default)]
pub struct Brunch {
    end: BlockId,
    blocks: Vec<BlockId>,
    is_loop: bool,
}

impl Brunch {
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
pub struct Loop {
    jmp: CndJmp,
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

/*
if 0000 {
    0035 +
} else {
    if 0024 { +
        003a +
        if 00ed { +
            0103 +
            00d8 +
            00c1 +
            00b7 +
            if 00ca { +
               00d5 +
               00e7 +
               0111
               004f
               006a
               if 0074 {
                  00a8,0054, 0129, 011a,00b7,0123,013e, 0061
               } else {
                  007d,0173,00b7,017e,00b7,
                  if 0189 {
                    01c2,0088,01cd,00b7,01d8,00b7,
                    if 01e3 {
                        0218, 0093, 0223, 00b7
                        if 022e {
                            0260, 00a0, 0074
                        } else {
                            0258, 0144
                        }
                    } else {
                      0210, 0144
                    }
                  } else {
                    01ba, 0144
                  }
               }
            } else {
               00d1
            }
        } else {
           00fb, 00b2
        }
    } else {
        0035
    }
}

blocks: [
CndBranch { jmp: CndJmp { block: 022e, true_br: 0260, false_br: 0258 }, true_br: Brunch { end: 0074, blocks: [0260, 00a0, 0074], is_loop: false }, false_br: Brunch { end: 0144, blocks: [0258, 0144], is_loop: false } },
CndBranch { jmp: CndJmp { block: 01e3, true_br: 0218, false_br: 0210 }, true_br: Brunch { end: 0144, blocks: [0218, 0093, 0223, 00b7, 022e, 0260, 00a0, 0074, 0258, 0144], is_loop: false }, false_br: Brunch { end: 0144, blocks: [0210, 0144], is_loop: false } },

CndBranch { jmp: CndJmp { block: 0189, true_br: 01c2, false_br: 01ba }, true_br: Brunch { end: 0144, blocks: [01c2, 0088, 01cd, 00b7, 01d8, 00b7, 01e3, 0218, 0093, 0223, 00b7, 022e, 0260, 00a0, 0074, 0258, 0144, 0210, 0144], is_loop: false }, false_br: Brunch { end: 0144, blocks: [01ba, 0144], is_loop: false } },

CndBranch { jmp: CndJmp { block: 0074, true_br: 00a8, false_br: 007d }, true_br: Brunch { end: 0061, blocks: [00a8, 0054, 0129, 011a, 00b7, 0123, 013e, 0061], is_loop: false }, false_br: Brunch { end: 0144, blocks: [007d, 0173, 00b7, 017e, 00b7, 0189, 01c2, 0088, 01cd, 00b7, 01d8, 00b7, 01e3, 0218, 0093, 0223, 00b7, 022e, 0260, 00a0, 0074, 0258, 0144, 0210, 0144, 01ba, 0144], is_loop: false } },
CndBranch { jmp: CndJmp { block: 00ca, true_br: 00d5, false_br: 00d1 }, true_br: Brunch { end: 0144, blocks: [00d5, 00e7, 0111, 004f, 006a, 0074, 00a8, 0054, 0129, 011a, 00b7, 0123, 013e, 0061, 007d, 0173, 00b7, 017e, 00b7, 0189, 01c2, 0088, 01cd, 00b7, 01d8, 00b7, 01e3, 0218, 0093, 0223, 00b7, 022e, 0260, 00a0, 0074, 0258, 0144, 0210, 0144, 01ba, 0144], is_loop: false }, false_br: Brunch { end: 00d1, blocks: [00d1], is_loop: false } },
CndBranch { jmp: CndJmp { block: 00ed, true_br: 0103, false_br: 00fb }, true_br: Brunch { end: 00d1, blocks: [0103, 00d8, 00c1, 00b7, 00ca, 00d5, 00e7, 0111, 004f, 006a, 0074, 00a8, 0054, 0129, 011a, 00b7, 0123, 013e, 0061, 007d, 0173, 00b7, 017e, 00b7, 0189, 01c2, 0088, 01cd, 00b7, 01d8, 00b7, 01e3, 0218, 0093, 0223, 00b7, 022e, 0260, 00a0, 0074, 0258, 0144, 0210, 0144, 01ba, 0144, 00d1], is_loop: false }, false_br: Brunch { end: 00b2, blocks: [00fb, 00b2], is_loop: false } },
CndBranch { jmp: CndJmp { block: 0024, true_br: 003a, false_br: 0035 }, true_br: Brunch { end: 00b2, blocks: [003a, 00ed, 0103, 00d8, 00c1, 00b7, 00ca, 00d5, 00e7, 0111, 004f, 006a, 0074, 00a8, 0054, 0129, 011a, 00b7, 0123, 013e, 0061, 007d, 0173, 00b7, 017e, 00b7, 0189, 01c2, 0088, 01cd, 00b7, 01d8, 00b7, 01e3, 0218, 0093, 0223, 00b7, 022e, 0260, 00a0, 0074, 0258, 0144, 0210, 0144, 01ba, 0144, 00d1, 00fb, 00b2], is_loop: false }, false_br: Brunch { end: 0035, blocks: [0035], is_loop: false } },
CndBranch { jmp: CndJmp { block: 0000, true_br: 0035, false_br: 0024 }, true_br: Brunch { end: 0035, blocks: [0035], is_loop: false }, false_br: Brunch { end: 0035, blocks: [0024, 003a, 00ed, 0103, 00d8, 00c1, 00b7, 00ca, 00d5, 00e7, 0111, 004f, 006a, 0074, 00a8, 0054, 0129, 011a, 00b7, 0123, 013e, 0061, 007d, 0173, 00b7, 017e, 00b7, 0189, 01c2, 0088, 01cd, 00b7, 01d8, 00b7, 01e3, 0218, 0093, 0223, 00b7, 022e, 0260, 00a0, 0074, 0258, 0144, 0210, 0144, 01ba, 0144, 00d1, 00fb, 00b2, 0035], is_loop: false } }]

 */
