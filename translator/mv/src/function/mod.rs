use crate::function::code::writer::FunctionCode;
use crate::function::signature::{map_signature, SignatureWriter};
use anyhow::Error;
use evm::function::FunDef as EthFunDef;
use move_binary_format::access::ModuleAccess;
use move_binary_format::file_format::{
    Bytecode, CodeUnit, FunctionDefinition, FunctionHandle, FunctionHandleIndex, IdentifierIndex,
    SignatureIndex, Visibility,
};
use move_binary_format::CompiledModule;
use move_core_types::identifier::Identifier;
use std::mem;

pub mod code;
pub mod signature;

#[derive(Debug)]
pub struct MvFunction {
    pub name: Identifier,
    pub visibility: Visibility,
    pub input: SignatureIndex,
    pub output: SignatureIndex,
    pub locals: SignatureIndex,
    pub code: Vec<Bytecode>,
}

impl MvFunction {
    pub fn new_public(
        def: EthFunDef,
        fn_code: FunctionCode,
        sign_writer: &mut SignatureWriter,
    ) -> Result<MvFunction, Error> {
        let input = sign_writer.make_signature(map_signature(def.abi.inputs.as_slice()));
        let output = sign_writer.make_signature(map_signature(def.abi.outputs.as_slice()));
        Ok(MvFunction {
            name: Identifier::new(&*def.abi.name)?,
            visibility: Visibility::Public,
            input,
            output,
            locals: sign_writer.make_signature(fn_code.locals),
            code: fn_code.code,
        })
    }

    pub fn write_function(mut self, module: &mut CompiledModule) -> Result<(), Error> {
        let name_id = IdentifierIndex(module.identifiers.len() as u16);
        module.identifiers.push(self.name.clone());
        let module_id = module.self_handle_idx();
        let handler = FunctionHandle {
            module: module_id,
            name: name_id,
            parameters: self.input,
            return_: self.output,
            type_parameters: vec![],
        };
        let handler_id = FunctionHandleIndex(module.function_handles.len() as u16);
        module.function_handles.push(handler);
        self.write_def(module, handler_id)?;
        Ok(())
    }

    fn write_def(
        &mut self,
        module: &mut CompiledModule,
        index: FunctionHandleIndex,
    ) -> Result<(), Error> {
        module.function_defs.push(FunctionDefinition {
            function: index,
            visibility: self.visibility,
            is_entry: matches!(self.visibility, Visibility::Public),
            acquires_global_resources: vec![],
            code: Some(CodeUnit {
                locals: self.locals,
                code: mem::take(&mut self.code),
            }),
        });
        Ok(())
    }
}
