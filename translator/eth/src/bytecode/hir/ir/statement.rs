use crate::bytecode::hir::ir::expression::Expr;
use crate::bytecode::hir::vars::VarId;
use crate::BlockId;

#[derive(Debug, Clone)]
pub enum Statement {
    Assign {
        var: VarId,
        expr: Expr,
    },
    MemStore8 {
        addr: Expr,
        var: Expr,
    },
    MemStore {
        addr: Expr,
        var: Expr,
    },
    SStore {
        addr: Expr,
        var: Expr,
    },
    Log {
        offset: Expr,
        len: Expr,
        topics: Vec<Expr>,
    },
    If {
        condition: Expr,
        true_branch: Vec<Statement>,
        false_branch: Vec<Statement>,
    },
    Loop {
        id: BlockId,
        condition_block: Vec<Statement>,
        condition: Expr,
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
        offset: Expr,
        len: Expr,
    },
}
