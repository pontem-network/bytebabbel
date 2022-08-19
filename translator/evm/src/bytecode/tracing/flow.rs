use crate::BlockId;

#[derive(Debug, Clone)]
pub enum Flow {
    Block(BlockId),
    Sequence(Vec<Flow>),
    If {
        cnd: BlockId,
        true_br: Box<Flow>,
        false_br: Box<Flow>,
    },
    Loop(Box<Flow>),
    Continue(BlockId),
    Break(BlockId),
    Stop,
}

impl Flow {
    pub fn is_stop(&self) -> bool {
        match self {
            Flow::Stop => true,
            Flow::Block(_) => false,
            Flow::Sequence(vec) => vec.last().map(Flow::is_stop).unwrap_or(false),
            Flow::If {
                cnd: _,
                true_br,
                false_br,
            } => true_br.is_stop() && false_br.is_stop(),
            Flow::Loop(_) => false,
            Flow::Continue(_) => true,
            Flow::Break(_) => true,
        }
    }
}
