use intrinsic::template;
use move_bytecode_verifier::CodeUnitVerifier;

#[test]
pub fn merge_mem_store() {
    CodeUnitVerifier::verify_module(&template()).unwrap();
}
