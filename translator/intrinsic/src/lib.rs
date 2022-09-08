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

pub enum Mem {
    Store,
    Store8,
    Load,
    Size,
    New,
}

impl Mem {
    pub fn token() -> SignatureToken {
        SignatureToken::Struct(StructHandleIndex(1))
    }

    pub fn func_handler(&self) -> FunctionHandleIndex {
        match self {
            Mem::Store => FunctionHandleIndex(13),
            Mem::Store8 => FunctionHandleIndex(14),
            Mem::Load => FunctionHandleIndex(12),
            Mem::Size => FunctionHandleIndex(6),
            Mem::New => FunctionHandleIndex(15),
        }
    }
}

pub enum Cast {
    AddressToNumber,
}

impl Cast {
    pub fn func_handler(&self) -> FunctionHandleIndex {
        match self {
            Cast::AddressToNumber => FunctionHandleIndex(0),
        }
    }
}

pub enum Storage {
    Load,
    Store,
    Create,
}

impl Storage {
    pub fn token() -> SignatureToken {
        SignatureToken::MutableReference(Box::new(SignatureToken::Struct(StructHandleIndex(2))))
    }

    pub fn instance() -> StructDefinitionIndex {
        StructDefinitionIndex(2)
    }

    pub fn func_handler(&self) -> FunctionHandleIndex {
        match self {
            Storage::Load => FunctionHandleIndex(27),
            Storage::Store => FunctionHandleIndex(29),
            Storage::Create => FunctionHandleIndex(10),
        }
    }
}

pub fn self_address_index() -> ConstantPoolIndex {
    ConstantPoolIndex(10)
}

pub enum Number {
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
}

impl Number {
    pub fn token() -> SignatureToken {
        SignatureToken::Struct(StructHandleIndex(3))
    }

    pub fn func_handler(&self) -> FunctionHandleIndex {
        match self {
            Number::Add => FunctionHandleIndex(20),
            Number::Sub => FunctionHandleIndex(20),
            Number::Mul => FunctionHandleIndex(19),
            Number::Div => FunctionHandleIndex(4),
            Number::Mod => FunctionHandleIndex(18),

            Number::BitOr => FunctionHandleIndex(16),
            Number::BitAnd => FunctionHandleIndex(17),
            Number::Xor => FunctionHandleIndex(18),
            Number::Shl => FunctionHandleIndex(19),
            Number::Shr => FunctionHandleIndex(20),
            Number::Lt => FunctionHandleIndex(21),
            Number::Gt => FunctionHandleIndex(22),
            Number::Le => FunctionHandleIndex(23),
            Number::Ge => FunctionHandleIndex(24),
            Number::Eq => FunctionHandleIndex(25),
            Number::Neq => FunctionHandleIndex(26),
        }
    }
}
