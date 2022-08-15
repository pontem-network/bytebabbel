use crate::bytecode::mir::ir::expression::Expression;
use crate::bytecode::mir::ir::types::LocalIndex;
use crate::bytecode::mir::translation::Variable;
use crate::BlockId;

#[derive(Debug)]
pub enum Statement {
    CreateVar(LocalIndex, Expression),
    SetVar(LocalIndex, Expression),
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
}
