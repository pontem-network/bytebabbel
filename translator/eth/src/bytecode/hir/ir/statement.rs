use crate::bytecode::hir::ir::var::VarId;
use crate::BlockId;

#[derive(Debug, Clone)]
pub enum Statement {
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
    Log {
        offset: VarId,
        len: VarId,
        topics: Vec<VarId>,
    },
    If {
        condition: VarId,
        true_branch: Vec<Statement>,
        false_branch: Vec<Statement>,
    },
    Loop {
        id: BlockId,
        condition_block: Vec<Statement>,
        condition: VarId,
        is_true_br_loop: bool,
        loop_br: Vec<Statement>,
    },
    Continue {
        loop_id: BlockId,
        context: Vec<Statement>,
    },
    Stop,
    Abort(u8),
    Result {
        offset: VarId,
        len: VarId,
    },
}
