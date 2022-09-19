use move_binary_format::file_format::{
    Constant, ConstantPoolIndex, FunctionHandleIndex, SignatureToken,
};
use move_binary_format::CompiledModule;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::CORE_CODE_ADDRESS;
use std::collections::HashSet;

pub mod table;

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

pub fn self_address_index() -> ConstantPoolIndex {
    ConstantPoolIndex(10)
}
