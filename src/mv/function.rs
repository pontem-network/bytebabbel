use crate::evm::bytecode::executor::BlockId;
use crate::evm::function::FunctionDefinition as EthFunDef;
use crate::evm::program::Program;
use crate::mv::function::signature::map_signature;
use anyhow::Error;
use move_binary_format::access::ModuleAccess;
use move_binary_format::file_format::{
    FunctionDefinition, FunctionHandle, FunctionHandleIndex, IdentifierIndex, Signature,
    SignatureIndex, Visibility,
};
use move_binary_format::CompiledModule;
use move_core_types::identifier::Identifier;

mod signature;

#[derive(Debug)]
pub struct MvFunction {
    pub name: Identifier,
    pub visibility: Visibility,
    pub entry_point: BlockId,
    pub input: Signature,
    pub output: Signature,
}

impl MvFunction {
    pub fn new_public(def: EthFunDef, _program: &Program) -> Result<MvFunction, Error> {
        Ok(MvFunction {
            name: Identifier::new(&*def.abi.name)?,
            visibility: Visibility::Public,
            entry_point: def.entry_point,
            input: map_signature(def.abi.inputs.as_slice()),
            output: map_signature(def.abi.outputs.as_slice()),
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
        let params_id = module.signatures.len();
        module.signatures.push(self.input.clone());
        let returns_id = module.signatures.len();
        module.signatures.push(self.output.clone());
        (
            SignatureIndex(params_id as u16),
            SignatureIndex(returns_id as u16),
        )
    }

    fn write_def(
        &self,
        module: &mut CompiledModule,
        index: FunctionHandleIndex,
    ) -> Result<(), Error> {
        module.function_defs.push(FunctionDefinition {
            function: index,
            visibility: self.visibility,
            is_entry: matches!(self.visibility, Visibility::Public),
            acquires_global_resources: vec![],
            code: None,
        });
        Ok(())
    }
}
