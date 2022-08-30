use anyhow::Error;
use move_binary_format::CompiledModule;

pub struct ModuleParts {}

impl ModuleParts {}

impl TryFrom<CompiledModule> for ModuleParts {
    type Error = Error;

    fn try_from(module: CompiledModule) -> Result<Self, Self::Error> {
        dbg!(module);

        todo!()
    }
}
