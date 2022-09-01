use crate::bytecode::hir::ir::var::VarId;
use crate::BlockId;

#[derive(Debug, Clone)]
pub enum Instruction {
    SetVar(VarId),
    MapVar {
        id: VarId,
        val: VarId,
    },
    MemStore8 {
        addr: VarId,
        var: VarId,
    },
    MemStore {
        addr: VarId,
        var: VarId,
    },
    SStore {
        addr: VarId,
        var: VarId,
    },
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
    Result {
        offset: VarId,
        len: VarId,
    },
}
