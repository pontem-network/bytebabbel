use move_binary_format::{
    CompiledModule,
    file_format::{FunctionHandleIndex, IdentifierIndex, FunctionDefinitionIndex, Bytecode},
    access::ModuleAccess,
};
use move_core_types::identifier::Identifier;

use crate::CodeUnit;

pub trait MoveFunction<'m>
where
    Self: Sized,
{
    fn name() -> String;

    fn eth_module(&self) -> &'m CompiledModule;

    fn new(eth_module: &'m CompiledModule) -> Self;

    fn call(self) -> CodeUnit {
        CodeUnit {
            code: vec![
                Bytecode::Call(self.function_handle_index())
            ],
        }
    }

    /// Should we handle *Loc opcodes in result bytecode?
    fn inline(&self) -> CodeUnit {
        let code = &self
            .eth_module()
            .function_def_at(self.function_definition_index())
            .code
            .as_ref()
            .unwrap();
        CodeUnit {
            code: code.code.clone(),
        }
    }

    fn function_handle_index(&self) -> FunctionHandleIndex {
        let identifier_index = &self
            .eth_module()
            .identifiers()
            .iter()
            .position(|name| name == &Identifier::new(Self::name()).unwrap())
            .map(|index| IdentifierIndex::new(index.try_into().unwrap()))
            .expect(format!("Unknown function name in MVIR module {}", Self::name()).as_str());
        self.eth_module()
            .function_handles()
            .iter()
            .position(|handle| handle.name == *identifier_index)
            .map(|index| FunctionHandleIndex::new(index.try_into().unwrap()))
            .expect(
                format!(
                    "Can't find HunctionHandle with IdentifierIndex({}) in CompiledModule",
                    identifier_index
                )
                .as_str(),
            )
    }

    fn function_definition_index(&self) -> FunctionDefinitionIndex {
        let fhi = self.function_handle_index();
        self.eth_module()
            .function_defs()
            .iter()
            .position(|definition| definition.function == fhi)
            .map(|index| FunctionDefinitionIndex::new(index.try_into().unwrap()))
            .expect(format!("Can't find HunctionDefinitionHandle with FunctionHandleIndex({}) in CompiledModule", fhi).as_str())
    }
}
