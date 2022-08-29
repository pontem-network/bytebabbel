use crate::bytecode::flow_graph::builder::CndJmp;
use crate::BlockId;

#[derive(Debug, Clone)]
pub enum Flow {
    Continue(BlockId),
    Block(BlockId),
    Loop(LoopFlow),
    IF(IfFlow),
    Sequence(Vec<Flow>),
}

#[derive(Debug, Clone)]
pub struct LoopFlow {
    pub jmp: CndJmp,
    pub br: LoopBr,
}

impl LoopFlow {
    pub fn break_block(&self) -> BlockId {
        if self.br.is_true_br_loop() {
            self.jmp.false_br
        } else {
            self.jmp.true_br
        }
    }
}

#[derive(Debug, Clone)]
pub enum LoopBr {
    TrueBr(Box<Flow>),
    FalseBr(Box<Flow>),
}

impl LoopBr {
    pub fn is_true_br_loop(&self) -> bool {
        matches!(self, LoopBr::TrueBr(_))
    }

    pub fn flow(&self) -> &Flow {
        match self {
            LoopBr::TrueBr(flow) => flow,
            LoopBr::FalseBr(flow) => flow,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IfFlow {
    pub jmp: CndJmp,
    pub true_br: Box<Flow>,
    pub false_br: Box<Flow>,
}
