use crate::evm::abi::EthType;
use move_binary_format::file_format::{Signature, SignatureToken};

pub fn map_signature(eth_types: &[EthType]) -> Signature {
    Signature(
        eth_types
            .iter()
            .map(|eth| match eth.tp.as_str() {
                "uint256" => SignatureToken::U128,
                _ => todo!(),
            })
            .collect(),
    )
}
