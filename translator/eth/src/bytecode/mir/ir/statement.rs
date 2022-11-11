use crate::bytecode::hir::ir::Label;
use crate::bytecode::loc::{Loc, Location};
use crate::bytecode::mir::ir::expression::TypedExpr;
use crate::bytecode::mir::translation::variables::Variable;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub enum Statement {
    InitStorage(Variable),
    StoreStack(BTreeMap<Variable, Loc<TypedExpr>>),
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
    Abort(u8),
    Result(Vec<Variable>),
    Log {
        storage: Variable,
        memory: Variable,
        offset: Loc<TypedExpr>,
        len: Loc<TypedExpr>,
        topics: Vec<Loc<TypedExpr>>,
    },
    Label(Label),
    BrTrue(Loc<TypedExpr>, Label),
    Br(Label),
    CallLocal {
        name: Label,
        storage: Variable,
        memory: Variable,
        context: Vec<Loc<TypedExpr>>,
    },
}

impl Statement {
    pub fn loc(self, loc: impl Location) -> Loc<Statement> {
        Loc::new(loc.start(), loc.end(), self)
    }
}
