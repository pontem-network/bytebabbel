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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence)]
pub enum Mem {
    New,
    Size,
    Load,
    Store,
    Store8,
    Hash,
}

impl Mem {
    pub fn token() -> SignatureToken {
        SignatureToken::Struct(StructHandleIndex(1))
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::New => "new_mem",
            Self::Size => "effective_len",
            Self::Load => "mload",
            Self::Store => "mstore",
            Self::Store8 => "mstore8",
            Self::Hash => "hash",
        }
    }

    pub fn func_handler(&self) -> FunctionHandleIndex {
        match self {
            Mem::New => FunctionHandleIndex(33),
            Mem::Size => FunctionHandleIndex(11),
            Mem::Load => FunctionHandleIndex(27),
            Mem::Store => FunctionHandleIndex(30),
            Mem::Store8 => FunctionHandleIndex(31),
            Mem::Hash => FunctionHandleIndex(21),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sequence)]
pub enum Storage {
    Create,
    Store,
    Load,
}

impl Storage {
    pub fn token() -> SignatureToken {
        SignatureToken::MutableReference(Box::new(SignatureToken::Struct(StructHandleIndex(2))))
    }

    pub fn instance() -> StructDefinitionIndex {
        StructDefinitionIndex(2)
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Create => "init_store",
            Self::Store => "sstore",
            Self::Load => "sload",
        }
    }

    pub fn func_handler(&self) -> FunctionHandleIndex {
        match self {
            Storage::Create => FunctionHandleIndex(22),
            Storage::Store => FunctionHandleIndex(49),
            Storage::Load => FunctionHandleIndex(47),
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
    BitOr,
    BitAnd,
    Xor,
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
    FromAddress,
    FromBytes,
    FromBool,
    ToBool,
}

impl Num {
    pub fn token() -> SignatureToken {
        SignatureToken::Struct(StructHandleIndex(3))
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Add => "overflowing_add",
            Self::Sub => "overflowing_sub",
            Self::Mul => "overflowing_mul",
            Self::Div => "div",
            Self::Mod => "mod",
            Self::BitOr => "bitor",
            Self::BitAnd => "bitand",
            Self::Xor => "bitxor",
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
        }
    }

    pub fn func_handler(&self) -> FunctionHandleIndex {
        match self {
            Num::Add => FunctionHandleIndex(34),
            Num::Sub => FunctionHandleIndex(37),
            Num::Mul => FunctionHandleIndex(36),
            Num::Div => FunctionHandleIndex(9),
            Num::Mod => FunctionHandleIndex(29),
            Num::BitOr => FunctionHandleIndex(4),
            Num::BitAnd => FunctionHandleIndex(2),
            Num::Xor => FunctionHandleIndex(6),
            Num::Shl => FunctionHandleIndex(43),
            Num::Shr => FunctionHandleIndex(45),
            Num::Lt => FunctionHandleIndex(26),
            Num::Gt => FunctionHandleIndex(20),
            Num::Le => FunctionHandleIndex(24),
            Num::Ge => FunctionHandleIndex(17),
            Num::Eq => FunctionHandleIndex(12),
            Num::Neq => FunctionHandleIndex(32),
            Num::BitNot => FunctionHandleIndex(3),
            Num::Byte => FunctionHandleIndex(7),
            Num::FromAddress => FunctionHandleIndex(13),
            Num::FromBytes => FunctionHandleIndex(15),
            Num::FromBool => FunctionHandleIndex(14),
            Num::ToBool => FunctionHandleIndex(50),
        }
    }
}
