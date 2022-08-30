pub mod parts;

use crate::parts::ModuleParts;
use anyhow::Error;
use move_binary_format::CompiledModule;

const MEM_MODULE: &[u8] = include_bytes!("../mv/build/intrinsic/bytecode_modules/mem.mv");
const STORAGE_MODULE: &[u8] = include_bytes!("../mv/build/intrinsic/bytecode_modules/store.mv");

pub fn mem_parts() -> Result<ModuleParts, Error> {
    let module = CompiledModule::deserialize(MEM_MODULE)?;
    ModuleParts::try_from(module)
}

pub fn storage_parts() -> Result<ModuleParts, Error> {
    let module = CompiledModule::deserialize(STORAGE_MODULE)?;
    ModuleParts::try_from(module)
}
