use crate::bytecode::flow_graph::builder::CndJmp;
use crate::BlockId;

#[derive(Debug, Clone)]
pub enum Flow {
    Block(BlockId),
    Loop(LoopFlow),
    IF(IfFlow),
    Sequence(Vec<Flow>),
}

#[derive(Debug, Clone)]
pub struct LoopFlow {
    pub loop_br: Box<Flow>,
}

#[derive(Debug, Clone)]
pub struct IfFlow {
    pub jmp: CndJmp,
    pub true_br: Box<Flow>,
    pub false_br: Box<Flow>,
}
