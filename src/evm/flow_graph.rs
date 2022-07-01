use crate::evm::{Instruction, Loc};
use move_binary_format::control_flow_graph::BlockId;
use std::collections::BTreeMap;

pub struct ControlFlowGraph {
    blocks: BTreeMap<BlockId, Loc<Vec<Instruction>>>,
}
