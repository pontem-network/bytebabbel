use crate::bytecode::executor::flow_graph::builder::{Branch, CndBranch};
use crate::bytecode::executor::flow_graph::debug::log_flow;
use crate::bytecode::executor::flow_graph::flow::{Flow, IfFlow, LoopFlow};
use crate::BlockId;
use std::collections::BTreeMap;

pub fn map_flow(elements: Vec<CndBranch>) -> Flow {
    if elements.is_empty() {
        return Flow::Sequence(vec![]);
    }
    let mut blocks = elements
        .into_iter()
        .map(|flow| (flow.block(), flow))
        .collect::<BTreeMap<BlockId, CndBranch>>();

    let first_block = blocks.keys().next().unwrap().clone();
    let first_block = blocks.remove(&first_block).unwrap();

    let flow = map_block(first_block, &mut blocks);
    log_flow(&flow);
    flow
}

fn map_block(mut block: CndBranch, branch_map: &BTreeMap<BlockId, CndBranch>) -> Flow {
    let common_tail = block.take_common_fail().into_iter().collect::<Vec<_>>();

    let mut seq = vec![];
    seq.push(Flow::IF(IfFlow {
        jmp: block.jmp,
        true_br: Box::new(map_if_brunch(&block.true_br, branch_map)),
        false_br: Box::new(map_if_brunch(&block.false_br, branch_map)),
    }));

    if !common_tail.is_empty() {
        seq.extend(map_branch(&common_tail, branch_map));
    }
    Flow::Sequence(seq)
}

fn map_if_brunch(branch: &Branch, branch_map: &BTreeMap<BlockId, CndBranch>) -> Flow {
    let blocks = &branch.blocks;
    if blocks.is_empty() {
        return Flow::Sequence(vec![]);
    }
    if branch.is_loop {
        Flow::Loop(LoopFlow {
            loop_br: Box::new(Flow::Sequence(map_branch(&blocks, branch_map))),
        })
    } else {
        Flow::Sequence(map_branch(&blocks, branch_map))
    }
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
            seq.push(map_block(element.clone(), elements));
            index += element.len();
        } else {
            index += 1;
            seq.push(Flow::Block(block));
        }
    }
    seq
}
