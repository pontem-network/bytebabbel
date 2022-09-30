use anyhow::{anyhow, Error};
use log::log_enabled;
use log::Level;
use move_binary_format::binary_views::BinaryIndexedView;
use move_binary_format::file_format::Signature;
use move_binary_format::CompiledModule;
use move_bytecode_source_map::mapping::SourceMapping;
use move_bytecode_verifier::{CodeUnitVerifier, VerifierConfig};
use move_disassembler::disassembler::Disassembler;
use move_disassembler::disassembler::DisassemblerOptions;
use move_ir_types::location::Spanned;

use crate::mv_ir::func::Func;

pub mod func;
pub mod interface;

#[derive(Debug)]
pub struct Module {
    funcs: Vec<Func>,
    signatures: Vec<Signature>,
    template: CompiledModule,
}

impl Module {
    pub fn new(funcs: Vec<Func>, signatures: Vec<Signature>, template: CompiledModule) -> Self {
        Self {
            funcs,
            signatures,
            template,
        }
    }

    pub fn make_move_module(self) -> Result<CompiledModule, Error> {
        let mut module = self.template;
        for func in self.funcs {
            func.write_function(&mut module)?;
        }
        module.signatures = self.signatures;

        print_move_module(&module);

        CodeUnitVerifier::verify_module(&VerifierConfig::default(), &module).map_err(|err| {
            anyhow!(
                "Verification error:{:?}-{:?}. Message:{:?}. Location: {:?} -{:?}",
                err.major_status(),
                err.sub_status(),
                err.message(),
                err.location(),
                err.indices()
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
