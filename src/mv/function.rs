use crate::evm::function::FunDef as EthFunDef;
use crate::mv::function::code::writer::FunctionCode;
use crate::mv::function::signature::map_signature;
use crate::mv::store_signatures;
use anyhow::Error;
use move_binary_format::access::ModuleAccess;
use move_binary_format::file_format::{
    Bytecode, CodeUnit, FunctionDefinition, FunctionHandle, FunctionHandleIndex, IdentifierIndex,
    Signature, SignatureIndex, SignatureToken, Visibility,
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
    pub input: Signature,
    pub output: Signature,
    pub locals: Vec<SignatureToken>,
    pub code: Vec<Bytecode>,
}

impl MvFunction {
    pub fn new_public(def: EthFunDef, fn_code: FunctionCode) -> Result<MvFunction, Error> {
        let input = map_signature(def.abi.inputs.as_slice());
        let output = map_signature(def.abi.outputs.as_slice());
        Ok(MvFunction {
            name: Identifier::new(&*def.abi.name)?,
            visibility: Visibility::Public,
            input,
            output,
            locals: fn_code.locals,
            code: fn_code.code,
        })
    }

    pub fn write_function(mut self, module: &mut CompiledModule) -> Result<(), Error> {
        let name_id = IdentifierIndex(module.identifiers.len() as u16);
        module.identifiers.push(self.name.clone());
        let module_id = module.self_handle_idx();
        let (params, returns) = self.write_signatures(module);
        let handler = FunctionHandle {
            module: module_id,
            name: name_id,
            parameters: params,
            return_: returns,
            type_parameters: vec![],
        };
        let handler_id = FunctionHandleIndex(module.function_handles.len() as u16);
        module.function_handles.push(handler);
        self.write_def(module, handler_id)?;
        Ok(())
    }

    fn write_signatures(
        &mut self,
        module: &mut CompiledModule,
    ) -> (SignatureIndex, SignatureIndex) {
        let input = store_signatures(module, mem::take(&mut self.input));
        let output = store_signatures(module, mem::take(&mut self.output));
        (input, output)
    }

    fn write_def(
        &mut self,
        module: &mut CompiledModule,
        index: FunctionHandleIndex,
    ) -> Result<(), Error> {
        let locals_id = store_signatures(module, Signature(mem::take(&mut self.locals)));

        module.function_defs.push(FunctionDefinition {
            function: index,
            visibility: self.visibility,
            is_entry: matches!(self.visibility, Visibility::Public),
            acquires_global_resources: vec![],
            code: Some(CodeUnit {
                locals: locals_id,
                code: mem::take(&mut self.code),
            }),
        });
        Ok(())
    }
}
