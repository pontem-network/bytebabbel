use eth::bytecode::types::EthType;
use eth::Flags;
use intrinsic::table::U256 as Num;
use move_binary_format::file_format::{Signature, SignatureIndex, SignatureToken};

pub fn map_signature(eth_types: &[EthType], is_native: bool, flags: &Flags) -> Vec<SignatureToken> {
    eth_types
        .iter()
        .map(|eth| map_type(eth, is_native, flags))
        .collect()
}

pub fn map_type(eth_type: &EthType, is_native: bool, flags: &Flags) -> SignatureToken {
    match eth_type {
        EthType::U256 => {
            if flags.u128_io {
                SignatureToken::U128
            } else {
                Num::token()
            }
        }
        EthType::Bool => SignatureToken::Bool,
        EthType::Address => {
            if is_native {
                SignatureToken::Address
            } else {
                signer()
            }
        }
        EthType::Bytes => SignatureToken::Vector(Box::new(SignatureToken::U8)),
    }
}

pub fn signer() -> SignatureToken {
    SignatureToken::Reference(Box::new(SignatureToken::Signer))
}

#[derive(Debug)]
pub struct SignatureWriter {
    signatures: Vec<Signature>,
}

impl SignatureWriter {
    pub fn new(signatures: &[Signature]) -> Self {
        Self {
            signatures: signatures.to_vec(),
        }
    }

    pub fn make_signature(&mut self, sign: Vec<SignatureToken>) -> SignatureIndex {
        let sign = Signature(sign);
        let idx = self
            .signatures
            .iter()
            .enumerate()
            .find(|(_, s)| s == &&sign)
            .map(|(i, _)| i);

        if let Some(idx) = idx {
            SignatureIndex(idx as u16)
        } else {
            let idx = SignatureIndex(self.signatures.len() as u16);
            self.signatures.push(sign);
            idx
        }
    }

    pub fn freeze(self) -> Vec<Signature> {
        self.signatures
    }
}
