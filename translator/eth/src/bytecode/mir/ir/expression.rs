use crate::bytecode::mir::ir::math::Operation;
use crate::bytecode::mir::ir::types::{LocalIndex, SType, Value};
use crate::bytecode::mir::translation::variables::Variable;
use anyhow::Error;

#[derive(Debug, Clone)]
pub enum Expression {
    GetMem,
    GetStore,
    MLoad { memory: Variable, offset: Variable },
    SLoad { storage: Variable, offset: Variable },
    MSize { memory: Variable },
    Const(Value),
    Var(Variable),
    Param(LocalIndex, SType),
    Operation(Operation, Variable, Variable),
    StackOps(StackOps),
}

#[derive(Debug, Clone)]
pub struct StackOps {
    pub vec: Vec<StackOp>,
}

#[derive(Debug, Clone)]
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
    pub fn push_var(mut self, var: Variable) -> StackOpsBuilder {
        self.stack.push(var.s_type());
        self.vec.push(StackOp::PushVar(var));
        self
    }

    pub fn push_const(mut self, var: Value) -> StackOpsBuilder {
        self.stack.push(var.s_type());
        self.vec.push(StackOp::PushConst(var));
        self
    }

    pub fn binary_op(
        mut self,
        op: Operation,
        ops: SType,
        res: SType,
    ) -> Result<StackOpsBuilder, Error> {
        let op1 = self
            .stack
            .pop()
            .ok_or_else(|| anyhow::anyhow!("stack is empty"))?;
        let op2 = self
            .stack
            .pop()
            .ok_or_else(|| anyhow::anyhow!("stack is empty"))?;
        if op1 != op2 || op1 != ops {
            return Err(anyhow::anyhow!(
                "incompatible types: binary_op({:?}) ops:{:?}, res:{:?}.\n{:?}",
                op,
                ops,
                res,
                self
            ));
        }
        self.vec.push(StackOp::BinaryOp(op));
        self.stack.push(res);
        Ok(self)
    }

    #[allow(clippy::should_implement_trait)]
    pub fn not(mut self) -> Result<StackOpsBuilder, Error> {
        let op = self
            .stack
            .pop()
            .ok_or_else(|| anyhow::anyhow!("stack is empty"))?;
        if op != SType::Bool {
            return Err(anyhow::anyhow!("incompatible types for not: {:?}", self));
        }
        self.vec.push(StackOp::Not);
        self.stack.push(SType::Bool);
        Ok(self)
    }

    pub fn build(mut self, tp: SType) -> Result<Expression, Error> {
        let res = self
            .stack
            .pop()
            .ok_or_else(|| anyhow::anyhow!("stack is empty"))?;
        if res != tp {
            return Err(anyhow::anyhow!(
                "incompatible result types:{:?}. Type:{:?}",
                self,
                tp
            ));
        }
        Ok(Expression::StackOps(StackOps { vec: self.vec }))
    }
}

impl Operation {
    pub fn expr(self, op1: Variable, op2: Variable) -> Expression {
        Expression::Operation(self, op1, op2)
    }
}
