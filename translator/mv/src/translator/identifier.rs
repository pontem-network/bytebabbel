use anyhow::Error;
use std::str::FromStr;

use move_binary_format::file_format::{IdentifierIndex, TableIndex};
use move_core_types::identifier::Identifier;

#[derive(Debug)]
pub struct IdentifierWriter {
    identifiers: Vec<Identifier>,
}

impl IdentifierWriter {
    pub fn new(identifiers: &[Identifier]) -> Self {
        Self {
            identifiers: identifiers.to_vec(),
        }
    }

    // TODO: change &str to Identifier
    pub fn make_identifier(&mut self, ident: &str) -> Result<IdentifierIndex, Error> {
        let ident = Identifier::from_str(ident)?;
        let idx = self.identifiers.iter().position(|s| s == &ident);

        if let Some(idx) = idx {
            Ok(IdentifierIndex(idx as TableIndex))
        } else {
            let idx = IdentifierIndex(self.identifiers.len() as TableIndex);
            self.identifiers.push(ident);
            Ok(idx)
        }
    }

    pub fn freeze(self) -> Vec<Identifier> {
        self.identifiers
    }
}
