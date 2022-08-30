pub mod func;

use crate::mv_ir::func::Func;
use anyhow::{anyhow, Error};
use log::log_enabled;
use log::Level;
use move_binary_format::binary_views::BinaryIndexedView;
use move_binary_format::file_format::{AddressIdentifierIndex, Signature};
use move_binary_format::internals::ModuleIndex;
use move_binary_format::CompiledModule;
use move_bytecode_source_map::mapping::SourceMapping;
use move_bytecode_verifier::{CodeUnitVerifier, VerifierConfig};
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_disassembler::disassembler::Disassembler;
use move_disassembler::disassembler::DisassemblerOptions;
use move_ir_types::location::Spanned;

#[derive(Debug)]
pub struct Module {
    address: AccountAddress,
    name: Identifier,
    funcs: Vec<Func>,
    signatures: Vec<Signature>,
    template: CompiledModule,
}

impl Module {
    pub fn new(
        address: AccountAddress,
        name: Identifier,
        funcs: Vec<Func>,
        signatures: Vec<Signature>,
        template: CompiledModule,
    ) -> Self {
        Self {
            address,
            name,
            funcs,
            signatures,
            template,
        }
    }

    pub fn make_move_module(self) -> Result<CompiledModule, Error> {
        let mut module = self.template;
        let existing_addr = module
            .address_identifiers
            .iter()
            .enumerate()
            .find(|(_, addr)| **addr == self.address);
        let index = if let Some((index, _)) = existing_addr {
            AddressIdentifierIndex(index as u16)
        } else {
            let index = module.address_identifiers.len() as u16;
            module.address_identifiers.push(self.address);
            AddressIdentifierIndex(index as u16)
        };

        module
            .module_handles
            .get_mut(module.self_module_handle_idx.0 as usize)
            .ok_or_else(|| anyhow!("Self module handle not found"))?
            .address = index;
        module.identifiers[module.self_module_handle_idx.into_index()] = self.name.clone();

        for func in self.funcs {
            func.write_function(&mut module)?;
        }

        module.signatures = self.signatures;

        print_move_module(&module);
        CodeUnitVerifier::verify_module(&VerifierConfig::default(), &module).map_err(|err| {
            anyhow!(
                "Verification error:{:?}-{:?}. Message:{:?}. Location: {:?}",
                err.major_status(),
                err.sub_status(),
                err.message(),
                err.location()
            )
        })?;
        Ok(module)
    }
}

fn print_move_module(module: &CompiledModule) {
    if !log_enabled!(Level::Trace) {
        return;
    }
    let source_mapping = SourceMapping::new_from_view(
        BinaryIndexedView::Module(module),
        Spanned::unsafe_no_loc(()).loc,
    )
    .unwrap();
    let disassembler = Disassembler::new(source_mapping, DisassemblerOptions::new());
    let dissassemble_string = disassembler.disassemble().unwrap();
    log::trace!("{}", dissassemble_string);
}
