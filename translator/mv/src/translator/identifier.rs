use anyhow::Error;
use std::str::FromStr;

use move_binary_format::file_format::{IdentifierIndex, TableIndex};
use move_core_types::identifier::Identifier;

#[derive(Debug)]
pub struct IdentidierWriter {
    identidiers: Vec<Identifier>,
}

impl IdentidierWriter {
    pub fn new(identidiers: &[Identifier]) -> Self {
        Self {
            identidiers: identidiers.to_vec(),
        }
    }

    // TODO: change &str to Idenifier
    pub fn make_identifier(&mut self, ident: &str) -> Result<IdentifierIndex, Error> {
        let ident = Identifier::from_str(ident)?;
        let idx = self.identidiers.iter().position(|s| s == &ident);

        if let Some(idx) = idx {
            Ok(IdentifierIndex(idx as TableIndex))
        } else {
            let idx = IdentifierIndex(self.identidiers.len() as TableIndex);
            self.identidiers.push(ident);
            Ok(idx)
        }
    }

    pub fn freeze(self) -> Vec<Identifier> {
        self.identidiers
    }
}
