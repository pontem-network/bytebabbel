use crate::bytecode::hir::ir::var::VarId;
use crate::{BlockId, U256};

#[derive(Debug, Clone)]
pub enum Instruction {
    SetVar(VarId),
    MapVar {
        id: VarId,
        val: VarId,
    },
    MemStore(U256, VarId),
    MemLoad(U256, VarId),
    If {
        condition: VarId,
        true_branch: Vec<Instruction>,
        false_branch: Vec<Instruction>,
    },
    Loop {
        id: BlockId,
        condition_block: Vec<Instruction>,
        condition: VarId,
        is_true_br_loop: bool,
        loop_br: Vec<Instruction>,
    },
    Continue {
        loop_id: BlockId,
        context: Vec<Instruction>,
    },
    Stop,
    Abort(u8),
    Result(Vec<VarId>),
}
