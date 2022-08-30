use move_binary_format::CompiledModule;

const TEMPLATE_MODULE: &[u8] = include_bytes!("../mv/build/intrinsic/bytecode_modules/template.mv");

pub fn template() -> CompiledModule {
    CompiledModule::deserialize(TEMPLATE_MODULE).unwrap()
}
