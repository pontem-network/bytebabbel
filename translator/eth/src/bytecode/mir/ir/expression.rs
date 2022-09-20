use crate::bytecode::hir::executor::math::{BinaryOp, TernaryOp, UnaryOp};
use crate::bytecode::mir::ir::types::{SType, Value};
use crate::bytecode::mir::translation::variables::Variable;
use anyhow::{anyhow, ensure, Error};

#[derive(Debug, Clone)]
pub enum Expression {
    GetMem,
    GetStore,
    MLoad {
        memory: Variable,
        offset: TypedExpr,
    },
    MSlice {
        memory: Variable,
        offset: TypedExpr,
        len: TypedExpr,
    },
    SLoad {
        storage: Variable,
        key: TypedExpr,
    },
    MSize {
        memory: Variable,
    },
    Const(Value),
    Var(Variable),
    UnOp(UnaryOp, TypedExpr),
    BinOp(BinaryOp, TypedExpr, TypedExpr),
    TernOp(TernaryOp, TypedExpr, TypedExpr, TypedExpr),
    StackOps(StackOps),
    Cast(TypedExpr, Cast),
    BytesLen(Variable),
    ReadNum {
        data: Variable,
        offset: TypedExpr,
    },
    Hash {
        mem: Variable,
        offset: TypedExpr,
        len: TypedExpr,
    },
}

impl Expression {
    pub fn ty(self, ty: SType) -> TypedExpr {
        TypedExpr::new(self, ty)
    }
}

#[derive(Debug, Clone)]
pub struct TypedExpr {
    pub expr: Box<Expression>,
    pub ty: SType,
}

impl TypedExpr {
    pub fn new(expr: Expression, ty: SType) -> Self {
        Self {
            expr: Box::new(expr),
            ty,
        }
    }

    pub fn as_var(&self) -> Option<Variable> {
        match &*self.expr {
            Expression::Var(var) => Some(var.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Cast {
    BoolToNum,
    SignerToNum,
    BytesToNum,
    NumToBool,
    AddressToNum,
    NumToAddress,
    RawNumToNum,
    NumToRawNum,
}

impl Cast {
    pub fn make(from: SType, to: SType) -> Result<Cast, Error> {
        match (from, to) {
            (SType::Bool, SType::Num) => Ok(Cast::BoolToNum),
            (SType::Signer, SType::Num) => Ok(Cast::SignerToNum),
            (SType::Bytes, SType::Num) => Ok(Cast::BytesToNum),
            (SType::Num, SType::Bool) => Ok(Cast::NumToBool),
            (SType::Address, SType::Num) => Ok(Cast::AddressToNum),
            (SType::Num, SType::Address) => Ok(Cast::NumToAddress),
            (SType::RawNum, SType::Num) => Ok(Cast::RawNumToNum),
            (SType::Num, SType::RawNum) => Ok(Cast::NumToRawNum),
            _ => Err(anyhow!("Can't cast {:?} to {:?}", from, to)),
        }
    }

    pub fn from(&self) -> SType {
        match self {
            Cast::BoolToNum => SType::Bool,
            Cast::SignerToNum => SType::Signer,
            Cast::BytesToNum => SType::Bytes,
            Cast::NumToBool => SType::Num,
            Cast::AddressToNum => SType::Address,
            Cast::NumToAddress => SType::Num,
            Cast::RawNumToNum => SType::RawNum,
            Cast::NumToRawNum => SType::Num,
        }
    }

    pub fn to(&self) -> SType {
        match self {
            Cast::BoolToNum => SType::Num,
            Cast::SignerToNum => SType::Num,
            Cast::BytesToNum => SType::Num,
            Cast::NumToBool => SType::Bool,
            Cast::AddressToNum => SType::Num,
            Cast::NumToAddress => SType::Address,
            Cast::RawNumToNum => SType::Num,
            Cast::NumToRawNum => SType::RawNum,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StackOps {
    pub vec: Vec<StackOp>,
}

#[derive(Debug, Clone)]
pub enum StackOp {
    PushBoolExpr(TypedExpr),
    PushBool(bool),
    Eq,
    Not,
}

#[derive(Default, Debug)]
pub struct StackOpsBuilder {
    stack: Vec<SType>,
    vec: Vec<StackOp>,
}

impl StackOpsBuilder {
    pub fn push_bool(mut self, var: TypedExpr) -> Result<StackOpsBuilder, Error> {
        ensure!(var.ty == SType::Bool, "Can't push bool from {:?}", var.ty);
        self.stack.push(var.ty);
        self.vec.push(StackOp::PushBoolExpr(var));
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

    pub fn push_const_bool(mut self, val: bool) -> StackOpsBuilder {
        self.stack.push(SType::Bool);
        self.vec.push(StackOp::PushBool(val));
        self
    }

    pub fn eq(mut self) -> Result<StackOpsBuilder, Error> {
        let op1 = self
            .stack
            .pop()
            .ok_or_else(|| anyhow::anyhow!("stack is empty"))?;
        let op2 = self
            .stack
            .pop()
            .ok_or_else(|| anyhow::anyhow!("stack is empty"))?;

        if op1 != op2 || op1 != SType::Bool {
            return Err(anyhow::anyhow!(
                "incompatible types: eq({:?}, {:?}):Bool.\n{:?}",
                op1,
                op2,
                self
            ));
        }
        self.vec.push(StackOp::Eq);
        self.stack.push(SType::Bool);
        Ok(self)
    }

    pub fn build(mut self, tp: SType) -> Result<TypedExpr, Error> {
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
        Ok(Expression::StackOps(StackOps { vec: self.vec }).ty(tp))
    }
}
