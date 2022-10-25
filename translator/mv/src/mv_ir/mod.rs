use anyhow::{anyhow, Error};
use log::{log_enabled, Level};
use move_binary_format::binary_views::BinaryIndexedView;
use move_binary_format::file_format::Signature;
use move_binary_format::{file_format::FunctionHandleIndex, CompiledModule};
use move_bytecode_source_map::mapping::SourceMapping;
use move_bytecode_verifier::{CodeUnitVerifier, VerifierConfig};
use move_disassembler::disassembler::Disassembler;
use move_disassembler::disassembler::DisassemblerOptions;
use move_ir_types::location::Spanned;

use crate::mv_ir::func::Func;

pub mod crop;
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

        let set = crop::find_all_functions(&module).unwrap();

        // generate vec of FunctionHandleIndexes to delete
        // TODO: transfer this code to crop.rs
        let mut indexes_to_delete: Vec<FunctionHandleIndex> = (0..module.function_handles.len())
            .map(|x| FunctionHandleIndex(x as u16))
            .collect();
        let mut vec_from_set: Vec<FunctionHandleIndex> = Vec::from_iter(set.into_iter());
        vec_from_set.sort();
        vec_from_set.reverse();
        for el in &vec_from_set {
            indexes_to_delete.remove(el.0 as usize);
        }

        let _ = crop::remove_function(&mut module, &indexes_to_delete);

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

pub fn print_move_module(module: &CompiledModule) {
    if !log_enabled!(Level::Trace) {
        return;
    }

    let source_mapping = SourceMapping::new_from_view(
        BinaryIndexedView::Module(module),
        Spanned::unsafe_no_loc(()).loc,
    )
    .unwrap();
    let mut ops = DisassemblerOptions::new();
    ops.only_externally_visible = true;
    let disassembler = Disassembler::new(source_mapping, ops);
    let dissassemble_string = disassembler.disassemble().unwrap();
    println!("{}", dissassemble_string);
}
