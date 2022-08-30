pub mod func;

use crate::mv_ir::func::Func;
use anyhow::{anyhow, Error};
use log::log_enabled;
use log::Level;
use move_binary_format::binary_views::BinaryIndexedView;
use move_binary_format::file_format::Signature;
use move_binary_format::internals::ModuleIndex;
use move_binary_format::CompiledModule;
use move_bytecode_source_map::mapping::SourceMapping;
use move_bytecode_verifier::CodeUnitVerifier;
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
        module.address_identifiers[module.self_module_handle_idx.into_index()] = self.address;
        module.identifiers[module.self_module_handle_idx.into_index()] = self.name.clone();

        dbg!(&module.address_identifiers);
        for func in self.funcs {
            func.write_function(&mut module)?;
        }

        module.signatures = self.signatures;

        print_move_module(&module);
        CodeUnitVerifier::verify_module(&module).map_err(|err| {
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
