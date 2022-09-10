use enum_iterator::Sequence;
use move_binary_format::file_format::{
    Constant, ConstantPoolIndex, FunctionHandleIndex, SignatureToken, StructDefinitionIndex,
    StructHandleIndex,
};
use move_binary_format::CompiledModule;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::CORE_CODE_ADDRESS;

pub const TEMPLATE_MODULE: &[u8] = include_bytes!("../mv/template.mv");

pub const SELF_ADDRESS_INDEX: ConstantPoolIndex = ConstantPoolIndex(4);

pub fn template(address: AccountAddress, name: &str) -> CompiledModule {
    let mut module = CompiledModule::deserialize(TEMPLATE_MODULE).unwrap();
    module.address_identifiers[0] = address;
    module.identifiers[0] = Identifier::new(name).unwrap();
    module.constant_pool[self_address_index().0 as usize] = Constant {
        type_: SignatureToken::Address,
        data: address.to_vec(),
    };

    if address == CORE_CODE_ADDRESS {
        module.address_identifiers.remove(1);
        for handle in &mut module.module_handles {
            if handle.address.0 == 1 {
                handle.address.0 = 0;
            }
        }
    }
    module
}

pub trait Function {
    fn name(&self) -> &'static str;
    fn handler(&self) -> FunctionHandleIndex;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence)]
pub enum Mem {
    New,
    Size,
    Load,
    Store,
    Store8,
    Slice,
    Hash,
    ReadRequestBuffer,
    RequestBufferLen,
}

impl Mem {
    pub fn token() -> SignatureToken {
        SignatureToken::Struct(StructHandleIndex(2))
    }
}

impl Function for Mem {
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
            Mem::ReadRequestBuffer => "read_request_buffer",
        }
    }

    fn handler(&self) -> FunctionHandleIndex {
        match self {
            Mem::New => FunctionHandleIndex(42),
            Mem::Size => FunctionHandleIndex(11),
            Mem::Load => FunctionHandleIndex(35),
            Mem::Store => FunctionHandleIndex(39),
            Mem::Store8 => FunctionHandleIndex(40),
            Mem::Hash => FunctionHandleIndex(23),
            Mem::Slice => FunctionHandleIndex(38),
            Mem::RequestBufferLen => FunctionHandleIndex(53),
            Mem::ReadRequestBuffer => FunctionHandleIndex(51),
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
        SignatureToken::MutableReference(Box::new(SignatureToken::Struct(StructHandleIndex(3))))
    }

    pub fn instance() -> StructDefinitionIndex {
        StructDefinitionIndex(3)
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
            Persist::InitContract => FunctionHandleIndex(24),
            Persist::Store => FunctionHandleIndex(67),
            Persist::Load => FunctionHandleIndex(63),
            Persist::Log0 => FunctionHandleIndex(29),
            Persist::Log1 => FunctionHandleIndex(30),
            Persist::Log2 => FunctionHandleIndex(31),
            Persist::Log3 => FunctionHandleIndex(32),
            Persist::Log4 => FunctionHandleIndex(33),
        }
    }
}

pub fn self_address_index() -> ConstantPoolIndex {
    ConstantPoolIndex(10)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence)]
pub enum Num {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    SDiv,
    SLt,
    SGt,
    SMod,
    Exp,
    SignExtend,
    BitOr,
    BitAnd,
    BitXor,
    Shl,
    Sar,
    Shr,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Neq,
    BitNot,
    Byte,
    FromAddress,
    FromBytes,
    FromBool,
    FromU64s,
    ToBool,
    IsZero,
}

impl Num {
    pub fn token() -> SignatureToken {
        SignatureToken::Struct(StructHandleIndex(4))
    }
}

impl Function for Num {
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
            Self::FromAddress => "from_address",
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
        }
    }

    fn handler(&self) -> FunctionHandleIndex {
        match self {
            Num::Add => FunctionHandleIndex(44),
            Num::Sub => FunctionHandleIndex(47),
            Num::Mul => FunctionHandleIndex(46),
            Num::Div => FunctionHandleIndex(9),
            Num::Mod => FunctionHandleIndex(37),
            Num::BitOr => FunctionHandleIndex(4),
            Num::BitAnd => FunctionHandleIndex(2),
            Num::BitXor => FunctionHandleIndex(6),
            Num::Shl => FunctionHandleIndex(59),
            Num::Shr => FunctionHandleIndex(61),
            Num::Lt => FunctionHandleIndex(34),
            Num::Gt => FunctionHandleIndex(22),
            Num::Le => FunctionHandleIndex(27),
            Num::Ge => FunctionHandleIndex(19),
            Num::Eq => FunctionHandleIndex(12),
            Num::Neq => FunctionHandleIndex(41),
            Num::BitNot => FunctionHandleIndex(3),
            Num::Byte => FunctionHandleIndex(7),
            Num::FromAddress => FunctionHandleIndex(14),
            Num::FromBytes => FunctionHandleIndex(16),
            Num::FromBool => FunctionHandleIndex(15),
            Num::ToBool => FunctionHandleIndex(68),
            Num::FromU64s => FunctionHandleIndex(18),
            Num::IsZero => FunctionHandleIndex(26),
            Num::SDiv => FunctionHandleIndex(56),
            Num::SLt => FunctionHandleIndex(64),
            Num::SGt => FunctionHandleIndex(58),
            Num::SMod => FunctionHandleIndex(65),
            Num::Exp => FunctionHandleIndex(13),
            Num::SignExtend => FunctionHandleIndex(57),
            Num::Sar => FunctionHandleIndex(55),
        }
    }
}
