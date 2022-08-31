use intrinsic::{template, Mem, Storage};
use move_binary_format::file_format::{
    FunctionHandleIndex, SignatureToken, StructDefinitionIndex, StructHandleIndex,
};
use move_bytecode_verifier::{CodeUnitVerifier, VerifierConfig};

#[test]
pub fn merge_mem_store() {
    CodeUnitVerifier::verify_module(&VerifierConfig::default(), &template()).unwrap();
}

#[test]
pub fn test_intrinsic_signature_token() {
    let template = template();
    let storage_token = Storage::token(&template);
    let mem_token = Mem::token(&template);
    let storage_func_index = Storage::instance(&template);
    assert_eq!(storage_func_index, StructDefinitionIndex(1));
    assert_eq!(
        storage_token,
        SignatureToken::MutableReference(Box::new(SignatureToken::Struct(StructHandleIndex(1))))
    );
    assert_eq!(
        mem_token,
        SignatureToken::MutableReference(Box::new(SignatureToken::Struct(StructHandleIndex(0))))
    );
    assert_eq!(
        mem_token,
        SignatureToken::MutableReference(Box::new(SignatureToken::Struct(StructHandleIndex(0))))
    );

    assert_eq!(Mem::New.func_handler(&template), FunctionHandleIndex(5));
    assert_eq!(Mem::Store.func_handler(&template), FunctionHandleIndex(3));
    assert_eq!(Mem::Store8.func_handler(&template), FunctionHandleIndex(4));
    assert_eq!(Mem::Load.func_handler(&template), FunctionHandleIndex(2));
    assert_eq!(Mem::Size.func_handler(&template), FunctionHandleIndex(0));

    assert_eq!(
        Storage::Store.func_handler(&template),
        FunctionHandleIndex(9)
    );
    assert_eq!(
        Storage::Load.func_handler(&template),
        FunctionHandleIndex(8)
    );
    assert_eq!(
        Storage::Create.func_handler(&template),
        FunctionHandleIndex(1)
    );
}
