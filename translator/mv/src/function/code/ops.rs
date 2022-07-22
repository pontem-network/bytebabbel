use evm::bytecode::executor::ops::BinaryOp;
use move_binary_format::file_format::{Bytecode, SignatureToken};

pub trait IntoCode {
    fn bytecode(&self) -> Vec<Bytecode>;
    fn signature_type(&self) -> SignatureToken;
    fn required_type(&self) -> Vec<SignatureToken>;
}

impl IntoCode for BinaryOp {
    fn bytecode(&self) -> Vec<Bytecode> {
        vec![match self {
            BinaryOp::EQ => Bytecode::Eq,
            BinaryOp::Lt => Bytecode::Lt,
            BinaryOp::Gt => Bytecode::Gt,
            BinaryOp::Shr => Bytecode::Shr,
            BinaryOp::Add => Bytecode::Add,
            BinaryOp::Mul => Bytecode::Mul,
            BinaryOp::Sub => Bytecode::Sub,
            BinaryOp::Div => Bytecode::Div,
            BinaryOp::SLt => Bytecode::Shl,
            BinaryOp::Byte => todo!(),
            BinaryOp::And => Bytecode::BitAnd,
        }]
    }

    fn signature_type(&self) -> SignatureToken {
        match self {
            BinaryOp::EQ | BinaryOp::Lt | BinaryOp::Gt => SignatureToken::Bool,
            BinaryOp::Shr => SignatureToken::U128,
            BinaryOp::Add => SignatureToken::U128,
            BinaryOp::Mul => SignatureToken::U128,
            BinaryOp::Sub => SignatureToken::U128,
            BinaryOp::Div => SignatureToken::U128,
            BinaryOp::SLt => SignatureToken::U128,
            BinaryOp::Byte => SignatureToken::U128,
            BinaryOp::And => SignatureToken::U128,
        }
    }

    fn required_type(&self) -> Vec<SignatureToken> {
        match self {
            BinaryOp::EQ => vec![SignatureToken::Bool, SignatureToken::U128],
            BinaryOp::Lt => vec![SignatureToken::U128],
            BinaryOp::Gt => vec![SignatureToken::U128],
            BinaryOp::Shr => vec![SignatureToken::U128],
            BinaryOp::Add => vec![SignatureToken::U128],
            BinaryOp::Mul => vec![SignatureToken::U128],
            BinaryOp::Sub => vec![SignatureToken::U128],
            BinaryOp::Div => vec![SignatureToken::U128],
            BinaryOp::SLt => vec![SignatureToken::U128],
            BinaryOp::Byte => vec![SignatureToken::U128],
            BinaryOp::And => vec![SignatureToken::U128],
        }
    }
}
