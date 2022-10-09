use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use std::collections::HashSet;
use std::sync::Mutex;

use move_binary_format::file_format::{
    Constant, ConstantPoolIndex, FunctionHandleIndex, SignatureToken,
};
use move_binary_format::{file_format::Visibility, CompiledModule};
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::CORE_CODE_ADDRESS;

pub mod table;

pub const TEMPLATE_MODULE: &[u8] = include_bytes!("../mv/template.mv");

pub const TOML_TEMPLATE: &str = include_str!("../mv/Move.toml");

pub const SELF_ADDRESS_INDEX: ConstantPoolIndex = ConstantPoolIndex(4);

lazy_static! {
    static ref CONST_ADDRES: Mutex<Option<u16>> = Mutex::new(None);
}

pub fn toml_template(name: &str, address: AccountAddress) -> String {
    TOML_TEMPLATE
        .replace("intrinsic", name)
        .replace("0x42", &address.to_hex_literal())
}

pub fn template(
    address: AccountAddress,
    name: &str,
    reserved_identifiers: &HashSet<String>,
) -> Result<CompiledModule> {
    let mut module = CompiledModule::deserialize(TEMPLATE_MODULE)?;
    module.address_identifiers[0] = address;
    module.identifiers[0] = Identifier::new(name)?;
    let const_address_index = self_address_index(&module)?.0;
    module.constant_pool[const_address_index as usize] = Constant {
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

    let public_functions = vec![
        table::U256::FromU128.handler(),
        table::U256::ToU128.handler(),
    ];
    for fun in &mut module.function_defs {
        if public_functions.contains(&fun.function) {
            (*fun).visibility = Visibility::Public;
            continue;
        }

        (*fun).visibility = Visibility::Private;
    }

    for ident in &mut module.identifiers {
        if reserved_identifiers.contains(ident.as_str()) {
            *ident = Identifier::new(format!("{}{}", ident, rand::random::<u16>()))?;
        }
    }

    Ok(module)
}

pub trait Function {
    fn name(&self) -> &'static str;
    fn handler(&self) -> FunctionHandleIndex;
}

pub fn self_address_index(module: &CompiledModule) -> Result<ConstantPoolIndex> {
    let mut cindex = CONST_ADDRES.lock().map_err(|err| anyhow!("{err}"))?;
    let val = match cindex.as_ref() {
        Some(data) => *data,
        None => {
            let const_index = find_address_const(&module, AccountAddress::from_hex_literal("0x42")?)
                .ok_or_else(|| anyhow!("The constant with the address 0x42 was not found"))?
                .0 as u16;

            *cindex = Some(const_index);
            const_index
        }
    };
    Ok(ConstantPoolIndex(val))
}

pub fn find_address_const(
    module: &CompiledModule,
    addr: AccountAddress,
) -> Option<ConstantPoolIndex> {
    module
        .constant_pool
        .iter()
        .enumerate()
        .find(|(_, c)| match c.type_ {
            SignatureToken::Address => c.data.as_slice() == addr.as_slice(),
            _ => false,
        })
        .map(|(id, _)| ConstantPoolIndex(id as u16))
}
