use crate::bytecode::hir::ir::Label;
use crate::bytecode::loc::Loc;
use crate::bytecode::mir::ir::expression::{Expression, TypedExpr};
use crate::bytecode::mir::translation::variables::Variable;
use crate::BlockId;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub enum Statement {
    InitStorage(Variable),
    StoreContext(BTreeMap<Variable, Loc<TypedExpr>>),
    Assign(Variable, Loc<TypedExpr>),
    MStore {
        memory: Variable,
        offset: Loc<TypedExpr>,
        val: Loc<TypedExpr>,
    },
    MStore8 {
        memory: Variable,
        offset: Loc<TypedExpr>,
        val: Loc<TypedExpr>,
    },
    SStore {
        storage: Variable,
        key: Loc<TypedExpr>,
        val: Loc<TypedExpr>,
    },
    IF {
        cnd: TypedExpr,
        true_br: Vec<Statement>,
        false_br: Vec<Statement>,
    },
    Loop {
        id: BlockId,
        cnd_calc: Vec<Statement>,
        cnd: Expression,
        // false br
        body: Vec<Statement>,
    },
    Abort(u8),
    Result(Vec<Variable>),
    Continue(BlockId),
    Log {
        storage: Variable,
        memory: Variable,
        offset: Loc<TypedExpr>,
        len: Loc<TypedExpr>,
        topics: Vec<Loc<TypedExpr>>,
    },
    Label(Label),
}
