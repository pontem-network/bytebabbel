use crate::bytecode::mir::ir::math::Operation;
use crate::bytecode::mir::ir::types::{SType, Value};
use crate::bytecode::mir::translation::Variable;
use anyhow::Error;

#[derive(Debug)]
pub enum Expression {
    Const(Value),
    Not(Variable),
    Var(Variable),
    Operation(Operation, Variable, Variable),
    StackOps(StackOps),
}

#[derive(Debug)]
pub struct StackOps {
    pub vec: Vec<StackOp>,
}

#[derive(Debug)]
pub enum StackOp {
    PushConst(Value),
    PushVar(Variable),
    BinaryOp(Operation),
    Not,
}

#[derive(Default, Debug)]
pub struct StackOpsBuilder {
    stack: Vec<SType>,
    vec: Vec<StackOp>,
}

impl StackOpsBuilder {
    pub fn push_var(&mut self, var: Variable) {
        self.stack.push(var.s_type());
        self.vec.push(StackOp::PushVar(var));
    }

    pub fn push_const(&mut self, var: Value) {
        self.stack.push(var.s_type());
        self.vec.push(StackOp::PushConst(var));
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

    pub fn build(mut self, tp: SType) -> Result<Expression, Error> {
        let res = self
            .stack
            .pop()
            .ok_or_else(|| anyhow::anyhow!("stack is empty"))?;
        if res != tp {
            return Err(anyhow::anyhow!("incompatible types"));
        }
        Ok(Expression::StackOps(StackOps { vec: self.vec }))
    }
}

impl Operation {
    pub fn expr(self, op1: Variable, op2: Variable) -> Expression {
        Expression::Operation(self, op1, op2)
    }
}
