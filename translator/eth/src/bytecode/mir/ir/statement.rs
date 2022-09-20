use crate::bytecode::mir::ir::expression::TypedExpr;
use crate::bytecode::mir::translation::variables::Variable;
use crate::BlockId;

#[derive(Debug, Clone)]
pub enum Statement {
    InitStorage(Variable),
    Assign(Variable, TypedExpr),
    MStore {
        memory: Variable,
        offset: TypedExpr,
        val: TypedExpr,
    },
    MStore8 {
        memory: Variable,
        offset: TypedExpr,
        val: TypedExpr,
    },
    SStore {
        storage: Variable,
        key: TypedExpr,
        val: TypedExpr,
    },
    IF {
        cnd: TypedExpr,
        true_br: Vec<Statement>,
        false_br: Vec<Statement>,
    },
    Loop {
        id: BlockId,
        cnd_calc: Vec<Statement>,
        cnd: TypedExpr,
        // false br
        body: Vec<Statement>,
    },
    Abort(u8),
    Result(Vec<TypedExpr>),
    Continue(BlockId),
    Log {
        storage: Variable,
        memory: Variable,
        offset: TypedExpr,
        len: TypedExpr,
        topics: Vec<TypedExpr>,
    },
}
