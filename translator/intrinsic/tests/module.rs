use intrinsic::{self_address_index, template, Cast, Mem, Storage};
use move_binary_format::access::ModuleAccess;
use move_binary_format::file_format::{
    Constant, ConstantPoolIndex, FunctionHandleIndex, SignatureToken, StructDefinitionIndex,
    StructHandleIndex,
};
use move_binary_format::CompiledModule;
use move_bytecode_verifier::{CodeUnitVerifier, VerifierConfig};
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::{ModuleId, CORE_CODE_ADDRESS};

#[test]
pub fn test_template_verification() {
    let address = AccountAddress::random();
    let template = template(address, "template_module");
    CodeUnitVerifier::verify_module(&VerifierConfig::default(), &template).unwrap();
}

#[test]
pub fn test_template_verification_core() {
    let template = template(CORE_CODE_ADDRESS, "template_module");
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
pub fn test_intrinsic_signature_token() {
    let address = AccountAddress::random();

    let template = template(address, "template_module");

    assert_eq!(
        template.self_id(),
        ModuleId::new(address, Identifier::new("template_module").unwrap())
    );

    assert_eq!(Storage::instance(), find_def(&template, "Persist"));
    assert_eq!(
        Storage::token(),
        SignatureToken::MutableReference(Box::new(SignatureToken::Struct(find_struct_by_name(
            &template, "Persist",
        ))))
    );
    assert_eq!(
        Mem::token(),
        SignatureToken::Struct(find_struct_by_name(&template, "Memory"))
    );

    assert_eq!(
        Mem::New.func_handler(),
        find_function_by_name(&template, "new_mem")
    );
    assert_eq!(
        Mem::Store.func_handler(),
        find_function_by_name(&template, "mstore")
    );
    assert_eq!(
        Mem::Store8.func_handler(),
        find_function_by_name(&template, "mstore8")
    );
    assert_eq!(
        Mem::Load.func_handler(),
        find_function_by_name(&template, "mload")
    );
    assert_eq!(
        Mem::Size.func_handler(),
        find_function_by_name(&template, "effective_len")
    );

    assert_eq!(
        Storage::Store.func_handler(),
        find_function_by_name(&template, "sstore")
    );
    assert_eq!(
        Storage::Load.func_handler(),
        find_function_by_name(&template, "sload")
    );
    assert_eq!(
        Storage::Create.func_handler(),
        find_function_by_name(&template, "init_store")
    );

    assert_eq!(
        Cast::AddressToNumber.func_handler(),
        find_function_by_name(&template, "address_to_number")
    );

    assert_eq!(self_address_index(), find_address_const(&template, address));
}

fn find_function_by_name(module: &CompiledModule, name: &str) -> FunctionHandleIndex {
    module
        .function_handles
        .iter()
        .enumerate()
        .find(|(_, h)| module.identifier_at(h.name).as_str() == name)
        .map(|(id, _)| FunctionHandleIndex(id as u16))
        .expect(format!("'{}' not found", name).as_str())
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
