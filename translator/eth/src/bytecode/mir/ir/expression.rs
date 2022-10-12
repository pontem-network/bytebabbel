use anyhow::{anyhow, Error};
use primitive_types::U256;

use crate::bytecode::hir::executor::math::{BinaryOp, TernaryOp, UnaryOp};
use crate::bytecode::loc::{Loc, Location};
use crate::bytecode::mir::ir::types::{SType, Value};
use crate::bytecode::mir::translation::variables::Variable;

#[derive(Debug, Clone)]
pub enum Expression {
    Const(Value),
    GetMem,
    GetStore,
    MLoad {
        memory: Variable,
        offset: Loc<TypedExpr>,
    },
    MSlice {
        memory: Variable,
        offset: Loc<TypedExpr>,
        len: Loc<TypedExpr>,
    },
    SLoad {
        storage: Variable,
        key: Loc<TypedExpr>,
    },
    MSize {
        memory: Variable,
    },
    MoveVar(Variable),
    CopyVar(Variable),
    Unary(UnaryOp, Loc<TypedExpr>),
    Binary(BinaryOp, Loc<TypedExpr>, Loc<TypedExpr>),
    Ternary(TernaryOp, Loc<TypedExpr>, Loc<TypedExpr>, Loc<TypedExpr>),
    Cast(Loc<TypedExpr>, Cast),
    BytesLen(Variable),
    ReadNum {
        data: Variable,
        offset: Loc<TypedExpr>,
    },
    Hash {
        mem: Variable,
        offset: Loc<TypedExpr>,
        len: Loc<TypedExpr>,
    },
    Balance {
        address: Variable,
    },
    GasPrice,
}

impl Expression {
    pub fn ty(self, ty: SType) -> TypedExpr {
        TypedExpr {
            expr: Box::new(self),
            ty,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypedExpr {
    pub expr: Box<Expression>,
    pub ty: SType,
}

impl TypedExpr {
    pub fn loc(self, loc: impl Location) -> Loc<TypedExpr> {
        Loc::new(loc.start(), loc.end(), self)
    }
}

impl From<U256> for TypedExpr {
    fn from(val: U256) -> Self {
        Expression::Const(Value::Number(val)).ty(SType::Num)
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
