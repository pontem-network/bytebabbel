use crate::bytecode::hir::executor::math::BinaryOp;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub enum Operation {
    Eq,
    Lt,
    Gt,
    Shr,
    Shl,
    Sar,
    Add,
    BitAnd,
    BitOr,
    BitXor,
    Mul,
    Sub,
    Div,
    Byte,
    Mod,
    SDiv,
    SLt,
    SGt,
    SMod,
    Exp,
    SignExtend,
    IsZero,
    BitNot,
}

impl From<BinaryOp> for Operation {
    fn from(op: BinaryOp) -> Self {
        match op {
            BinaryOp::EQ => Operation::Eq,
            BinaryOp::Lt => Operation::Lt,
            BinaryOp::Gt => Operation::Gt,
            BinaryOp::Shr => Operation::Shr,
            BinaryOp::Shl => Operation::Shl,
            BinaryOp::Sar => Operation::Sar,
            BinaryOp::Add => Operation::Add,
            BinaryOp::And => Operation::BitAnd,
            BinaryOp::Or => Operation::BitOr,
            BinaryOp::Xor => Operation::BitXor,
            BinaryOp::Mul => Operation::Mul,
            BinaryOp::Sub => Operation::Sub,
            BinaryOp::Div => Operation::Div,
            BinaryOp::Byte => Operation::Byte,
            BinaryOp::Mod => Operation::Mod,
            BinaryOp::SDiv => Operation::SDiv,
            BinaryOp::SLt => Operation::SLt,
            BinaryOp::SGt => Operation::SGt,
            BinaryOp::SMod => Operation::SMod,
            BinaryOp::Exp => Operation::Exp,
            BinaryOp::SignExtend => Operation::SignExtend,
        }
    }
}
