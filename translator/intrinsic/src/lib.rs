use move_binary_format::access::ModuleAccess;
use move_binary_format::file_format::{
    FunctionHandleIndex, SignatureToken, StructDefinitionIndex, StructHandleIndex,
};
use move_binary_format::CompiledModule;

const TEMPLATE_MODULE: &[u8] = include_bytes!("../mv/build/intrinsic/bytecode_modules/template.mv");

pub fn template() -> CompiledModule {
    CompiledModule::deserialize(TEMPLATE_MODULE).unwrap()
}

pub enum Mem {
    Store,
    Store8,
    Load,
    Size,
    New,
}

impl Mem {
    pub fn token(module: &CompiledModule) -> SignatureToken {
        SignatureToken::MutableReference(Box::new(SignatureToken::Struct(find_struct_by_name(
            module, "Memory",
        ))))
    }

    pub fn func_handler(&self, module: &CompiledModule) -> FunctionHandleIndex {
        let name = match self {
            Mem::Store => "mstore",
            Mem::Store8 => "mstore8",
            Mem::Load => "mload",
            Mem::Size => "effective_len",
            Mem::New => "new",
        };
        find_function_by_name(module, name)
    }
}

pub enum Storage {
    Load,
    Store,
    Create,
}

impl Storage {
    pub fn token(module: &CompiledModule) -> SignatureToken {
        SignatureToken::MutableReference(Box::new(SignatureToken::Struct(find_struct_by_name(
            module, "Persist",
        ))))
    }

    pub fn instance(module: &CompiledModule) -> StructDefinitionIndex {
        let id = find_struct_by_name(module, "Persist");
        module
            .struct_defs
            .iter()
            .enumerate()
            .find(|(_, def)| def.struct_handle == id)
            .map(|(id, _)| StructDefinitionIndex(id as u16))
            .unwrap()
    }

    pub fn func_handler(&self, module: &CompiledModule) -> FunctionHandleIndex {
        let name = match self {
            Storage::Load => "sload",
            Storage::Store => "sstore",
            Storage::Create => "init_store",
        };
        find_function_by_name(module, name)
    }
}

fn find_function_by_name(module: &CompiledModule, name: &str) -> FunctionHandleIndex {
    module
        .function_handles
        .iter()
        .enumerate()
        .find(|(_, h)| module.identifier_at(h.name).as_str() == name)
        .map(|(id, _)| FunctionHandleIndex(id as u16))
        .unwrap()
}

fn find_struct_by_name(module: &CompiledModule, name: &str) -> StructHandleIndex {
    module
        .struct_handles
        .iter()
        .enumerate()
        .find(|(_, h)| {
            let res = &module.identifiers[h.name.0 as usize];
            res.as_str() == name
        })
        .map(|(i, _)| StructHandleIndex(i as u16))
        .unwrap()
}
