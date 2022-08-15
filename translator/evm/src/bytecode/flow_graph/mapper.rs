use crate::bytecode::flow_graph::builder::CndBranch;
use crate::bytecode::flow_graph::debug::log_flow;
use crate::bytecode::flow_graph::flow::{Flow, IfFlow, LoopFlow};
use crate::bytecode::flow_graph::LoopBr;
use crate::BlockId;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
struct Mapper {
    blocks: BTreeMap<BlockId, CndBranch>,
    /// continue -> loop head
    loop_map: BTreeMap<BlockId, BlockId>,
}

impl Mapper {
    pub fn new(elements: Vec<CndBranch>) -> Mapper {
        let blocks = elements
            .into_iter()
            .map(|flow| (flow.block(), flow))
            .collect::<BTreeMap<BlockId, CndBranch>>();
        Mapper {
            blocks,
            loop_map: Default::default(),
        }
    }

    pub fn make_flow(mut self) -> Flow {
        if self.blocks.is_empty() {
            return Flow::Sequence(vec![]);
        }

        let first_block = *self.blocks.keys().next().unwrap();
        let first_block = self.blocks.remove(&first_block).unwrap();

        let flow = self.map_branch(first_block);
        log_flow(&flow);
        flow
    }

    fn update_loop_map(&mut self, block: &mut CndBranch) {
        if let Some(ctn) = block.true_br.continue_blocks.take() {
            self.loop_map.insert(ctn.continue_block, ctn.loop_head);
        }

        if let Some(ctn) = block.false_br.continue_blocks.take() {
            self.loop_map.insert(ctn.continue_block, ctn.loop_head);
        }
    }

    fn map_branch(&mut self, mut block: CndBranch) -> Flow {
        let mut seq = vec![];
        self.update_loop_map(&mut block);
        match (block.true_br.is_loop, block.false_br.is_loop) {
            (true, true) => {
                unreachable!("Loop in two branches")
            }
            (true, false) => {
                seq.push(Flow::Loop(LoopFlow {
                    jmp: block.jmp,
                    br: LoopBr::TrueBr(Box::new(Flow::Sequence(
                        self.map_block(&block.true_br.blocks),
                    ))),
                }));
                if !block.false_br.blocks.is_empty() {
                    seq.extend(self.map_block(&block.false_br.blocks));
                }
            }
            (false, true) => {
                seq.push(Flow::Loop(LoopFlow {
                    jmp: block.jmp,
                    br: LoopBr::FalseBr(Box::new(Flow::Sequence(
                        self.map_block(&block.false_br.blocks),
                    ))),
                }));
                if !block.true_br.blocks.is_empty() {
                    seq.extend(self.map_block(&block.true_br.blocks));
                }
            }
            (false, false) => {
                // IF
                let common_tail = block.take_common_fail().into_iter().collect::<Vec<_>>();

                seq.push(Flow::IF(IfFlow {
                    jmp: block.jmp,
                    true_br: Box::new(Flow::Sequence(self.map_block(&block.true_br.blocks))),
                    false_br: Box::new(Flow::Sequence(self.map_block(&block.false_br.blocks))),
                }));
                if !common_tail.is_empty() {
                    seq.extend(self.map_block(&common_tail));
                }
            }
        }
        Flow::Sequence(seq)
    }

    fn map_block(&mut self, blocks: &[BlockId]) -> Vec<Flow> {
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
            if let Some(element) = self.blocks.get(&block) {
                index += element.len();
                seq.push(self.map_branch(element.clone()));
            } else {
                index += 1;
                if let Some(block) = self.loop_map.get(&block) {
                    seq.push(Flow::Continue(*block));
                } else {
                    seq.push(Flow::Block(block));
                }
            }
        }
        seq
    }
}

pub fn map_flow(elements: Vec<CndBranch>) -> Flow {
    let mapper = Mapper::new(elements);
    mapper.make_flow()
}
