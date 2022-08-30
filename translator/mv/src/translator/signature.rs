use evm::abi::inc_ret_param::types::ParamType;
use evm::abi::inc_ret_param::Param as EthType;
use move_binary_format::file_format::{Signature, SignatureIndex, SignatureToken};

pub fn map_signature(eth_types: &[EthType]) -> Vec<SignatureToken> {
    eth_types
        .iter()
        .map(|eth| match eth.tp {
            ParamType::UInt(_) => SignatureToken::U128,
            ParamType::Bool => SignatureToken::Bool,
            _ => todo!(),
        })
        .collect()
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
