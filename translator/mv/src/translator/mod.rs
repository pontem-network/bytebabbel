use crate::mv_ir::func::Func;
use crate::mv_ir::Module;
use crate::translator::signature::{map_signature, SignatureWriter};
use anyhow::{anyhow, Error};
use evm::bytecode::mir::ir::Mir;
use evm::function::FunDef;
use evm::program::Program;
use move_binary_format::file_format::{Bytecode, SignatureIndex, Visibility};
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;

pub mod signature;

#[derive(Default)]
pub struct MvIrTranslator {
    sign_writer: SignatureWriter,
}

impl MvIrTranslator {
    pub fn translate(mut self, address: AccountAddress, program: Program) -> Result<Module, Error> {
        let name = Identifier::new(program.name())?;

        let funcs = program
            .public_functions()
            .into_iter()
            .map(|def| self.translate_func(def, &program))
            .collect::<Result<_, _>>()?;

        Ok(Module {
            address,
            name,
            funcs,
            signatures: self.sign_writer.freeze(),
        })
    }

    fn translate_func(&mut self, def: FunDef, program: &Program) -> Result<Func, Error> {
        let name = Identifier::new(def.abi.name().as_deref().unwrap_or("anonymous"))?;
        let visibility = Visibility::Public;
        let input = self
            .sign_writer
            .make_signature(map_signature(def.abi.inputs().unwrap().as_slice()));

        let output = self
            .sign_writer
            .make_signature(map_signature(def.abi.outputs().unwrap().as_slice()));

        let mir = program
            .function_mir(def.hash)
            .ok_or_else(|| anyhow!("Function {} not found", def.hash))?;

        let (locals, code) = self.translate_mir(mir)?;

        Ok(Func {
            name,
            visibility,
            input,
            output,
            locals,
            code,
        })
    }

    fn translate_mir(&mut self, mir: &Mir) -> Result<(SignatureIndex, Vec<Bytecode>), Error> {
        todo!()
    }
}
