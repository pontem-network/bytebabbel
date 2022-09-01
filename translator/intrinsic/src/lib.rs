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
        SignatureToken::Struct(StructHandleIndex(0))
    }

    pub fn func_handler(&self) -> FunctionHandleIndex {
        match self {
            Mem::Store => FunctionHandleIndex(3),
            Mem::Store8 => FunctionHandleIndex(4),
            Mem::Load => FunctionHandleIndex(2),
            Mem::Size => FunctionHandleIndex(0),
            Mem::New => FunctionHandleIndex(5),
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
        SignatureToken::MutableReference(Box::new(SignatureToken::Struct(StructHandleIndex(1))))
    }

    pub fn instance() -> StructDefinitionIndex {
        StructDefinitionIndex(1)
    }

    pub fn func_handler(&self) -> FunctionHandleIndex {
        match self {
            Storage::Load => FunctionHandleIndex(8),
            Storage::Store => FunctionHandleIndex(9),
            Storage::Create => FunctionHandleIndex(1),
        }
    }
}

pub fn self_address_index() -> ConstantPoolIndex {
    ConstantPoolIndex(4)
}
