use crate::bytecode::mir::ir::math::Operation;
use crate::bytecode::mir::ir::types::{LocalIndex, SType, Value};
use crate::bytecode::mir::translation::Variable;
use crate::BlockId;
use anyhow::Error;

#[derive(Debug)]
pub enum Statement {
    Const(Value),
    Not(Variable),
    CreateVar(LocalIndex, Box<Statement>),
    MemStore(Variable, Variable),
    Operation(Operation, Variable, Variable),
    IF {
        cnd: VarOrStack,
        true_br: Vec<Statement>,
        false_br: Vec<Statement>,
    },
    Loop {
        id: BlockId,
        cnd_calc: Vec<Statement>,
        cnd: VarOrStack,
        // false br
        body: Vec<Statement>,
    },
    StackOps(StackOps),
    Abort(u8),
    Result(Vec<Variable>),
    Continue(BlockId),
}

#[derive(Debug)]
pub enum VarOrStack {
    Var(Variable),
    Stack(StackOps),
}

#[derive(Debug)]
pub struct StackOps {
    pub vec: Vec<StackOp>,
}

#[derive(Debug)]
pub enum StackOp {
    Push(Variable),
    BinaryOp(Operation),
    Not,
}

#[derive(Default, Debug)]
pub struct StackOpsBuilder {
    stack: Vec<SType>,
    vec: Vec<StackOp>,
}

impl StackOpsBuilder {
    pub fn push(&mut self, var: Variable) {
        self.stack.push(var.s_type());
        self.vec.push(StackOp::Push(var));
    }
    pub fn binary_op(&mut self, op: Operation, ops: SType, res: SType) -> Result<(), Error> {
        let op1 = self
            .stack
            .pop()
            .ok_or_else(|| anyhow::anyhow!("stack is empty"))?;
        let op2 = self
            .stack
            .pop()
            .ok_or_else(|| anyhow::anyhow!("stack is empty"))?;
        if op1 != op2 || op1 != ops {
            return Err(anyhow::anyhow!("incompatible types"));
        }
        self.vec.push(StackOp::BinaryOp(op));
        self.stack.push(res);
        Ok(())
    }

    pub fn not(&mut self) -> Result<(), Error> {
        let op = self
            .stack
            .pop()
            .ok_or_else(|| anyhow::anyhow!("stack is empty"))?;
        if op != SType::Bool {
            return Err(anyhow::anyhow!("incompatible types"));
        }
        self.vec.push(StackOp::Not);
        self.stack.push(SType::Bool);
        Ok(())
    }

    pub fn build(mut self, tp: SType) -> Result<StackOps, Error> {
        let res = self
            .stack
            .pop()
            .ok_or_else(|| anyhow::anyhow!("stack is empty"))?;
        if res != tp {
            return Err(anyhow::anyhow!("incompatible types"));
        }
        Ok(StackOps { vec: self.vec })
    }
}
