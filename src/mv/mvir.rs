use crate::evm::program::Program;
use crate::mv::function::MvFunction;
use anyhow::Error;
use move_binary_format::file_format::empty_module;
use move_binary_format::internals::ModuleIndex;
use move_binary_format::CompiledModule;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;

#[derive(Debug)]
pub struct MvModule {
    address: AccountAddress,
    name: Identifier,
    funcs: Vec<MvFunction>,
}

impl MvModule {
    pub fn from_evm_program(address: AccountAddress, program: Program) -> Result<MvModule, Error> {
        let funcs = program
            .public_functions()
            .into_iter()
            .map(|def| MvFunction::new_public(def, &program))
            .collect::<Result<_, _>>()?;

        Ok(MvModule {
            address,
            name: Identifier::new(program.name())?,
            funcs,
        })
    }

    pub fn make_move_module(&self) -> Result<CompiledModule, Error> {
        let mut module = self.empty_module();
        for func in &self.funcs {
            func.write_function(&mut module)?;
        }
        Ok(module)
    }

    fn empty_module(&self) -> CompiledModule {
        let mut module = empty_module();
        module
            .address_identifiers
            .remove(module.self_module_handle_idx.into_index());
        module
            .identifiers
            .remove(module.self_module_handle_idx.into_index());
        module
            .address_identifiers
            .insert(module.self_module_handle_idx.into_index(), self.address);
        module.identifiers.insert(
            module.self_module_handle_idx.into_index(),
            self.name.clone(),
        );
        module
    }
}
