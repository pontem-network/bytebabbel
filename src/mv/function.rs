use crate::evm::bytecode::executor::BlockId;
use crate::evm::function::FunctionDefinition;
use crate::evm::program::Program;
use anyhow::Error;
use move_binary_format::file_format::Visibility;
use move_core_types::identifier::Identifier;

#[derive(Debug)]
pub struct MvFunction {
    pub name: Identifier,
    pub visibility: Visibility,
    pub is_entry: bool,
    pub entry_point: BlockId,
}

impl MvFunction {
    pub fn new_public(def: FunctionDefinition, _program: &Program) -> Result<MvFunction, Error> {
        Ok(MvFunction {
            name: Identifier::new(&*def.abi.name)?,
            visibility: Visibility::Public,
            is_entry: false,
            entry_point: def.entry_point,
        })
    }
}
