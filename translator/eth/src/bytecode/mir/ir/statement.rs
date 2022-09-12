use crate::bytecode::mir::ir::expression::Expression;
use crate::bytecode::mir::translation::variables::Variable;
use crate::BlockId;

#[derive(Debug, Clone)]
pub enum Statement {
    InitStorage(Variable),
    Assign(Variable, Expression),
    MStore {
        memory: Variable,
        offset: Variable,
        val: Variable,
    },
    MStore8 {
        memory: Variable,
        offset: Variable,
        val: Variable,
    },
    SStore {
        storage: Variable,
        key: Variable,
        val: Variable,
    },
    IF {
        cnd: Expression,
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
        offset: Variable,
        len: Variable,
        topics: Vec<Variable>,
    },
}
