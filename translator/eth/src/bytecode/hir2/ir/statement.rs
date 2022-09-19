use crate::bytecode::hir2::ir::expression::Expr;
use crate::bytecode::hir2::vars::VarId;
use crate::BlockId;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Statement {
    Assign {
        var: VarId,
        expr: Rc<Expr>,
    },
    MemStore8 {
        addr: Rc<Expr>,
        var: Rc<Expr>,
    },
    MemStore {
        addr: Rc<Expr>,
        var: Rc<Expr>,
    },
    SStore {
        addr: Rc<Expr>,
        var: Rc<Expr>,
    },
    Log {
        offset: Rc<Expr>,
        len: Rc<Expr>,
        topics: Vec<Rc<Expr>>,
    },
    If {
        condition: Rc<Expr>,
        true_branch: Vec<Statement>,
        false_branch: Vec<Statement>,
    },
    Loop {
        id: BlockId,
        condition_block: Vec<Statement>,
        condition: Rc<Expr>,
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
        offset: Rc<Expr>,
        len: Rc<Expr>,
    },
    Move(Rc<Expr>),
}
