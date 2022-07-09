use crate::evm::function::FunctionDefinition as EthFunDef;
use crate::evm::program::Program;
use crate::mv::function::code::MvIr;
use crate::mv::function::signature::map_signature;
use crate::mv::store_signatures;
use anyhow::Error;
use move_binary_format::access::ModuleAccess;
use move_binary_format::file_format::{
    CodeUnit, FunctionDefinition, FunctionHandle, FunctionHandleIndex, IdentifierIndex, Signature,
    SignatureIndex, Visibility,
};
use move_binary_format::CompiledModule;
use move_core_types::identifier::Identifier;

mod code;
mod signature;

#[derive(Debug)]
pub struct MvFunction {
    pub name: Identifier,
    pub visibility: Visibility,
    pub input: Signature,
    pub output: Signature,
    pub code: MvIr,
}

impl MvFunction {
    pub fn new_public(def: EthFunDef, program: &Program) -> Result<MvFunction, Error> {
        Ok(MvFunction {
            name: Identifier::new(&*def.abi.name)?,
            visibility: Visibility::Public,
            input: map_signature(def.abi.inputs.as_slice()),
            output: map_signature(def.abi.outputs.as_slice()),
            code: MvIr::make_ir(def, program)?,
        })
    }

    pub fn write_function(&self, module: &mut CompiledModule) -> Result<(), Error> {
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

    fn write_signatures(&self, module: &mut CompiledModule) -> (SignatureIndex, SignatureIndex) {
        let input = store_signatures(module, self.input.clone());
        let output = store_signatures(module, self.output.clone());
        (input, output)
    }

    fn write_def(
        &self,
        module: &mut CompiledModule,
        index: FunctionHandleIndex,
    ) -> Result<(), Error> {
        let locals_id = store_signatures(module, Signature(self.code.locals()));

        module.function_defs.push(FunctionDefinition {
            function: index,
            visibility: self.visibility,
            is_entry: matches!(self.visibility, Visibility::Public),
            acquires_global_resources: vec![],
            code: Some(CodeUnit {
                locals: locals_id,
                code: self.code.bytecode()?,
            }),
        });
        Ok(())
    }
}
