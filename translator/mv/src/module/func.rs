use anyhow::Error;
use intrinsic::table::Persist;
use move_binary_format::access::ModuleAccess;
use move_binary_format::file_format::{
    Bytecode, CodeUnit, FunctionDefinition, FunctionHandle, FunctionHandleIndex, IdentifierIndex,
    SignatureIndex, Visibility,
};
use move_binary_format::CompiledModule;
use move_core_types::identifier::Identifier;
use std::mem;

#[derive(Debug)]
pub struct Func {
    pub name: Identifier,
    pub visibility: Visibility,
    pub input: SignatureIndex,
    pub output: SignatureIndex,
    pub locals: SignatureIndex,
    pub code: Vec<Bytecode>,
}

impl Func {
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
            is_entry: true,
            acquires_global_resources: vec![Persist::instance()],
            code: Some(CodeUnit {
                locals: self.locals,
                code: mem::take(&mut self.code),
            }),
        });
        Ok(())
    }
}
