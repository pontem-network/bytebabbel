use enum_iterator::all;

#[cfg(test)]
use std::collections::HashSet;

use intrinsic::table::{Memory as Mem, Persist, U256 as Num};
use intrinsic::{self_address_index, template, Function};

use move_binary_format::access::ModuleAccess;
use move_binary_format::file_format::{
    Constant, ConstantPoolIndex, FunctionHandleIndex, SignatureToken, StructDefinitionIndex,
    StructHandleIndex, Visibility,
};
use move_binary_format::CompiledModule;
use move_bytecode_verifier::{CodeUnitVerifier, VerifierConfig};
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{ModuleId, CORE_CODE_ADDRESS};

#[test]
pub fn test_template_verification() {
    let address = AccountAddress::random();

    let template = template(
        address,
        "template_module",
        &["div".to_string(), "mod".to_string()]
            .into_iter()
            .collect::<HashSet<_>>(),
    );
    CodeUnitVerifier::verify_module(&VerifierConfig::default(), &template).unwrap();
}

#[test]
pub fn test_template_verification_core() {
    let template = template(
        CORE_CODE_ADDRESS,
        "template_module",
        &["sstore".to_string(), "mod".to_string()]
            .into_iter()
            .collect(),
    );
    CodeUnitVerifier::verify_module(&VerifierConfig::default(), &template).unwrap();
}

#[test]
pub fn test_template() {
    let module = CompiledModule::deserialize(intrinsic::TEMPLATE_MODULE).unwrap();
    assert_eq!(
        module.self_id(),
        ModuleId::new(
            AccountAddress::from_hex_literal("0x42").unwrap(),
            Identifier::new("template").unwrap()
        )
    );
    assert_eq!(
        module.constant_pool[self_address_index().0 as usize],
        Constant {
            type_: SignatureToken::Address,
            data: AccountAddress::from_hex_literal("0x42").unwrap().to_vec(),
        }
    );
}

#[test]
pub fn test_intrinsic_signature_token_mem_store() {
    let address = AccountAddress::random();

    let template = template(address, "template_module", &HashSet::new());

    assert_eq!(
        template.self_id(),
        ModuleId::new(address, Identifier::new("template_module").unwrap())
    );

    assert_eq!(Persist::instance(), find_def(&template, "Persist"));
    assert_eq!(
        Persist::token(),
        SignatureToken::MutableReference(Box::new(SignatureToken::Struct(find_struct_by_name(
            &template, "Persist",
        ))))
    );
    assert_eq!(
        Mem::token(),
        SignatureToken::Struct(find_struct_by_name(&template, "Memory"))
    );

    assert_eq!(self_address_index(), find_address_const(&template, address));

    assert_eq!(
        Num::token(),
        SignatureToken::Struct(find_struct_by_name(&template, "U256"))
    );
}

#[test]
pub fn test_intrinsic_function_visibility() {
    let address = AccountAddress::random();
    let template = template(address, "template_module", &HashSet::new());
    let public_functions = vec![Num::FromU128.handler(), Num::ToU128.handler()];

    for fun in &template.function_defs {
        if public_functions.contains(&fun.function) {
            assert_eq!(fun.visibility, Visibility::Public);
        } else {
            assert_eq!(fun.visibility, Visibility::Private);
        }
    }
}

#[test]
pub fn test_intrinsic_signature_token() {
    let address = AccountAddress::random();
    let template = template(address, "template_module", &HashSet::new());

    let diff: Vec<Mem> = all::<Mem>()
        .filter(|mem| find_function_by_name(&template, mem.name()) != mem.handler())
        .collect();

    for mem in &diff {
        println!(
            "{} -> {:?}",
            mem.name(),
            find_function_by_name(&template, mem.name())
        );
    }
    if !diff.is_empty() {
        panic!("Some functions are not found");
    }

    let diff: Vec<Persist> = all::<Persist>()
        .filter(|store| find_function_by_name(&template, store.name()) != store.handler())
        .collect();

    for store in &diff {
        println!(
            "{} -> {:?}",
            store.name(),
            find_function_by_name(&template, store.name())
        );
    }
    if !diff.is_empty() {
        panic!("Some functions are not found");
    }

    let diff: Vec<Num> = all::<Num>()
        .filter(|num| find_function_by_name(&template, num.name()) != num.handler())
        .collect();

    for num in &diff {
        println!(
            "{} {} -> {:?}",
            num.name(),
            num.handler().0,
            find_function_by_name(&template, num.name())
        );
    }
    if !diff.is_empty() {
        panic!("Some functions are not found");
    }
}

fn find_function_by_name(module: &CompiledModule, name: &str) -> FunctionHandleIndex {
    module
        .function_handles
        .iter()
        .enumerate()
        .find(|(_, h)| module.identifier_at(h.name).as_str() == name)
        .map(|(id, _)| FunctionHandleIndex(id as u16))
        .unwrap_or_else(|| panic!("'{}' not found", name))
}

fn find_struct_by_name(module: &CompiledModule, name: &str) -> StructHandleIndex {
    module
        .struct_handles
        .iter()
        .enumerate()
        .find(|(_, h)| {
            let res = &module.identifiers[h.name.0 as usize];
            res.as_str() == name
        })
        .map(|(i, _)| StructHandleIndex(i as u16))
        .unwrap()
}

fn find_def(module: &CompiledModule, name: &str) -> StructDefinitionIndex {
    let id = find_struct_by_name(module, name);
    module
        .struct_defs
        .iter()
        .enumerate()
        .find(|(_, def)| def.struct_handle == id)
        .map(|(id, _)| StructDefinitionIndex(id as u16))
        .unwrap()
}

fn find_address_const(module: &CompiledModule, addr: AccountAddress) -> ConstantPoolIndex {
    module
        .constant_pool
        .iter()
        .enumerate()
        .find(|(_, c)| match c.type_ {
            SignatureToken::Address => c.data.as_slice() == addr.as_slice(),
            _ => false,
        })
        .map(|(id, _)| ConstantPoolIndex(id as u16))
        .unwrap()
}
