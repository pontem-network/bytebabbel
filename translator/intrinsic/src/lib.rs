use enum_iterator::Sequence;
use move_binary_format::file_format::{
    Constant, ConstantPoolIndex, FunctionHandleIndex, SignatureToken, StructDefinitionIndex,
    StructHandleIndex,
};
use move_binary_format::CompiledModule;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::CORE_CODE_ADDRESS;
use std::collections::HashSet;

pub const TEMPLATE_MODULE: &[u8] = include_bytes!("../mv/template.mv");

pub const TOML_TEMPLATE: &str = include_str!("../mv/Move.toml");

pub const SELF_ADDRESS_INDEX: ConstantPoolIndex = ConstantPoolIndex(4);

pub fn toml_template(name: &str, address: AccountAddress) -> String {
    TOML_TEMPLATE
        .replace("intrinsic", name)
        .replace("0x42", &address.to_hex_literal())
}

pub fn template(
    address: AccountAddress,
    name: &str,
    reserved_identifiers: &HashSet<String>,
) -> CompiledModule {
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

    for ident in &mut module.identifiers {
        if reserved_identifiers.contains(ident.as_str()) {
            *ident = Identifier::new(format!("{}{}", ident, rand::random::<u16>())).unwrap();
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
            Mem::New => FunctionHandleIndex(43),
            Mem::Size => FunctionHandleIndex(11),
            Mem::Load => FunctionHandleIndex(36),
            Mem::Store => FunctionHandleIndex(40),
            Mem::Store8 => FunctionHandleIndex(41),
            Mem::Hash => FunctionHandleIndex(24),
            Mem::Slice => FunctionHandleIndex(39),
            Mem::RequestBufferLen => FunctionHandleIndex(54),
            Mem::ReadRequestBuffer => FunctionHandleIndex(52),
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
            Persist::InitContract => FunctionHandleIndex(25),
            Persist::Store => FunctionHandleIndex(68),
            Persist::Load => FunctionHandleIndex(64),
            Persist::Log0 => FunctionHandleIndex(30),
            Persist::Log1 => FunctionHandleIndex(31),
            Persist::Log2 => FunctionHandleIndex(32),
            Persist::Log3 => FunctionHandleIndex(33),
            Persist::Log4 => FunctionHandleIndex(34),
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
    FromSigner,
    FromAddress,
    FromBytes,
    FromBool,
    FromU64s,
    FromU128,
    ToBool,
    ToAddress,
    ToU128,
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
            Num::Add => FunctionHandleIndex(45),
            Num::Sub => FunctionHandleIndex(48),
            Num::Mul => FunctionHandleIndex(47),
            Num::Div => FunctionHandleIndex(9),
            Num::Mod => FunctionHandleIndex(38),
            Num::BitOr => FunctionHandleIndex(4),
            Num::BitAnd => FunctionHandleIndex(2),
            Num::BitXor => FunctionHandleIndex(6),
            Num::Shl => FunctionHandleIndex(60),
            Num::Shr => FunctionHandleIndex(62),
            Num::Lt => FunctionHandleIndex(35),
            Num::Gt => FunctionHandleIndex(23),
            Num::Le => FunctionHandleIndex(28),
            Num::Ge => FunctionHandleIndex(20),
            Num::Eq => FunctionHandleIndex(12),
            Num::Neq => FunctionHandleIndex(42),
            Num::BitNot => FunctionHandleIndex(3),
            Num::Byte => FunctionHandleIndex(7),
            Num::FromSigner => FunctionHandleIndex(17),
            Num::FromBytes => FunctionHandleIndex(16),
            Num::FromBool => FunctionHandleIndex(15),
            Num::ToBool => FunctionHandleIndex(70),
            Num::FromU64s => FunctionHandleIndex(19),
            Num::IsZero => FunctionHandleIndex(27),
            Num::SDiv => FunctionHandleIndex(57),
            Num::SLt => FunctionHandleIndex(65),
            Num::SGt => FunctionHandleIndex(59),
            Num::SMod => FunctionHandleIndex(66),
            Num::Exp => FunctionHandleIndex(13),
            Num::SignExtend => FunctionHandleIndex(58),
            Num::Sar => FunctionHandleIndex(56),
            Num::FromAddress => FunctionHandleIndex(14),
            Num::ToAddress => FunctionHandleIndex(69),
            Num::FromU128 => FunctionHandleIndex(18),
            Num::ToU128 => FunctionHandleIndex(0),
        }
    }
}
