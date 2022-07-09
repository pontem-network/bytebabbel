use move_binary_format::file_format::{Signature, SignatureIndex};
use move_binary_format::CompiledModule;

pub mod function;
pub mod mvir;

pub fn store_signatures(module: &mut CompiledModule, sing: Signature) -> SignatureIndex {
    let index = module
        .signatures
        .iter()
        .enumerate()
        .find(|(_, s)| *s == &sing)
        .map(|(i, _)| i);
    if let Some(index) = index {
        SignatureIndex(index as u16)
    } else {
        let params_id = module.signatures.len() as u16;
        module.signatures.push(sing);
        SignatureIndex(params_id)
    }
}
