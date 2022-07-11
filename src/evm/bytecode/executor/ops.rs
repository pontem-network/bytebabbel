use std::fmt::{Display, Formatter};

#[derive(Hash, Eq, PartialEq, Debug)]
pub enum BinaryOp {
    EQ,
    Lt,
    Gt,
    Shr,
    Add,
    And,
    Mul,
    Sub,
    Div,
    SLt,
    Byte,
}

#[derive(Hash, Eq, PartialEq, Debug)]
pub enum UnaryOp {
    IsZero,
    Not,
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
