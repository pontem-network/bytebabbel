use crate::Function;
use enum_iterator::Sequence;
use move_binary_format::file_format::{
    FunctionHandleIndex, SignatureToken, StructDefinitionIndex, StructHandleIndex,
};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence)]
pub enum Memory {
    New,
    Size,
    Load,
    Store,
    Store8,
    Hash,
    Slice,
    RequestBufferLen,
    ReadRequestBuffer,
}

impl Memory {
    pub fn token() -> SignatureToken {
        SignatureToken::Struct(StructHandleIndex(1))
    }

    pub fn instance() -> StructDefinitionIndex {
        StructDefinitionIndex(1)
    }
}

impl Function for Memory {
    fn name(&self) -> &'static str {
        match self {
            Self::New => "new_mem",
            Self::Size => "effective_len",
            Self::Load => "mload",
            Self::Store => "mstore",
            Self::Store8 => "mstore8",
            Self::Hash => "hash",
            Self::Slice => "mslice",
            Self::RequestBufferLen => "request_buffer_len",
            Self::ReadRequestBuffer => "read_request_buffer",
        }
    }

    fn handler(&self) -> FunctionHandleIndex {
        match self {
            Self::New => FunctionHandleIndex(48),
            Self::Size => FunctionHandleIndex(13),
            Self::Load => FunctionHandleIndex(39),
            Self::Store => FunctionHandleIndex(44),
            Self::Store8 => FunctionHandleIndex(45),
            Self::Hash => FunctionHandleIndex(27),
            Self::Slice => FunctionHandleIndex(43),
            Self::RequestBufferLen => FunctionHandleIndex(64),
            Self::ReadRequestBuffer => FunctionHandleIndex(62),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence)]
pub enum Persist {
    InitContract,
    Store,
    Load,
    Log0,
    Log1,
    Log2,
    Log3,
    Log4,
}

impl Persist {
    pub fn token() -> SignatureToken {
        SignatureToken::MutableReference(Box::new(SignatureToken::Struct(StructHandleIndex(2))))
    }

    pub fn instance() -> StructDefinitionIndex {
        StructDefinitionIndex(2)
    }
}

impl Function for Persist {
    fn name(&self) -> &'static str {
        match self {
            Self::InitContract => "init_contract",
            Self::Store => "sstore",
            Self::Load => "sload",
            Self::Log0 => "log0",
            Self::Log1 => "log1",
            Self::Log2 => "log2",
            Self::Log3 => "log3",
            Self::Log4 => "log4",
        }
    }

    fn handler(&self) -> FunctionHandleIndex {
        match self {
            Self::InitContract => FunctionHandleIndex(28),
            Self::Store => FunctionHandleIndex(80),
            Self::Load => FunctionHandleIndex(76),
            Self::Log0 => FunctionHandleIndex(33),
            Self::Log1 => FunctionHandleIndex(34),
            Self::Log2 => FunctionHandleIndex(35),
            Self::Log3 => FunctionHandleIndex(36),
            Self::Log4 => FunctionHandleIndex(37),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence)]
pub enum U256 {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    BitOr,
    BitAnd,
    BitXor,
    Shl,
    Shr,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Neq,
    BitNot,
    Byte,
    FromSigner,
    FromBytes,
    FromBool,
    ToBool,
    FromU64s,
    IsZero,
    SDiv,
    SLt,
    SGt,
    SMod,
    Exp,
    SignExtend,
    Sar,
    FromAddress,
    ToAddress,
    FromU128,
    ToU128,
}

impl U256 {
    pub fn token() -> SignatureToken {
        SignatureToken::Struct(StructHandleIndex(3))
    }

    pub fn instance() -> StructDefinitionIndex {
        StructDefinitionIndex(3)
    }
}

impl Function for U256 {
    fn name(&self) -> &'static str {
        match self {
            Self::Add => "overflowing_add",
            Self::Sub => "overflowing_sub",
            Self::Mul => "overflowing_mul",
            Self::Div => "div",
            Self::Mod => "mod",
            Self::BitOr => "bitor",
            Self::BitAnd => "bitand",
            Self::BitXor => "bitxor",
            Self::Shl => "shl",
            Self::Shr => "shr",
            Self::Lt => "lt",
            Self::Gt => "gt",
            Self::Le => "le",
            Self::Ge => "ge",
            Self::Eq => "eq",
            Self::Neq => "ne",
            Self::BitNot => "bitnot",
            Self::Byte => "byte",
            Self::FromSigner => "from_signer",
            Self::FromBytes => "from_bytes",
            Self::FromBool => "from_bool",
            Self::ToBool => "to_bool",
            Self::FromU64s => "from_u64s",
            Self::IsZero => "is_zero",
            Self::SDiv => "sdiv",
            Self::SLt => "slt",
            Self::SGt => "sgt",
            Self::SMod => "smod",
            Self::Exp => "exp",
            Self::SignExtend => "sexp",
            Self::Sar => "sar",
            Self::FromAddress => "from_address",
            Self::ToAddress => "to_address",
            Self::FromU128 => "from_u128",
            Self::ToU128 => "as_u128",
        }
    }

    fn handler(&self) -> FunctionHandleIndex {
        match self {
            Self::Add => FunctionHandleIndex(52),
            Self::Sub => FunctionHandleIndex(57),
            Self::Mul => FunctionHandleIndex(55),
            Self::Div => FunctionHandleIndex(12),
            Self::Mod => FunctionHandleIndex(41),
            Self::BitOr => FunctionHandleIndex(5),
            Self::BitAnd => FunctionHandleIndex(3),
            Self::BitXor => FunctionHandleIndex(8),
            Self::Shl => FunctionHandleIndex(70),
            Self::Shr => FunctionHandleIndex(73),
            Self::Lt => FunctionHandleIndex(38),
            Self::Gt => FunctionHandleIndex(26),
            Self::Le => FunctionHandleIndex(31),
            Self::Ge => FunctionHandleIndex(22),
            Self::Eq => FunctionHandleIndex(14),
            Self::Neq => FunctionHandleIndex(47),
            Self::BitNot => FunctionHandleIndex(4),
            Self::Byte => FunctionHandleIndex(9),
            Self::FromSigner => FunctionHandleIndex(19),
            Self::FromBytes => FunctionHandleIndex(18),
            Self::FromBool => FunctionHandleIndex(17),
            Self::ToBool => FunctionHandleIndex(82),
            Self::FromU64s => FunctionHandleIndex(21),
            Self::IsZero => FunctionHandleIndex(30),
            Self::SDiv => FunctionHandleIndex(67),
            Self::SLt => FunctionHandleIndex(77),
            Self::SGt => FunctionHandleIndex(69),
            Self::SMod => FunctionHandleIndex(78),
            Self::Exp => FunctionHandleIndex(15),
            Self::SignExtend => FunctionHandleIndex(68),
            Self::Sar => FunctionHandleIndex(66),
            Self::FromAddress => FunctionHandleIndex(16),
            Self::ToAddress => FunctionHandleIndex(81),
            Self::FromU128 => FunctionHandleIndex(20),
            Self::ToU128 => FunctionHandleIndex(1),
        }
    }
}
