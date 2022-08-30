use intrinsic::template;
use move_bytecode_verifier::{CodeUnitVerifier, VerifierConfig};

#[test]
pub fn merge_mem_store() {
    CodeUnitVerifier::verify_module(&VerifierConfig::default(), &template()).unwrap();
}
