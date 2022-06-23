use move_binary_format::CompiledModule;

use crate::translator::MoveFunction;

#[derive(Debug)]
pub struct AddMod<'m> {
    eth_module: &'m CompiledModule,
}

impl<'m> MoveFunction<'m> for AddMod<'m> {
    fn new(eth_module: &'m CompiledModule) -> Self {
        Self { eth_module }
    }

    fn name() -> String {
        "addmod".to_owned()
    }

    fn eth_module(&self) -> &'m CompiledModule {
        self.eth_module
    }
}

#[derive(Debug)]
pub struct MulMod<'m> {
    eth_module: &'m CompiledModule,
}

impl<'m> MoveFunction<'m> for MulMod<'m> {
    fn new(eth_module: &'m CompiledModule) -> Self {
        Self { eth_module }
    }

    fn name() -> String {
        "mulmod".to_owned()
    }

    fn eth_module(&self) -> &'m CompiledModule {
        self.eth_module
    }
}

// @TODO: Overload MVIR module with constant test implementation
#[cfg(test)]
mod test {
    use std::path::Path;

    use move_ir_compiler::util::do_compile_module;

    use crate::translator::ETH_MODULE_PATH;
    use super::*;

    #[test]
    fn test_addmod_inline() {
        let module = do_compile_module(Path::new(ETH_MODULE_PATH), &[]).0;
        let addmod = AddMod::new(&module);
        // assert_eq!(addmod.function_handle_index(), FunctionHandleIndex::new(0));
        dbg!(&addmod);
        dbg!(&addmod.inline());
    }

    #[test]
    fn test_addmod_call() {
        let module = do_compile_module(Path::new(ETH_MODULE_PATH), &[]).0;
        let addmod = AddMod::new(&module);
        // assert_eq!(addmod.function_handle_index(), FunctionHandleIndex::new(0));
        dbg!(&addmod);
        dbg!(&addmod.call());
    }

    #[test]
    fn test_mulmod_inline() {
        let module = do_compile_module(Path::new(ETH_MODULE_PATH), &[]).0;
        let addmod = MulMod::new(&module);
        // assert_eq!(addmod.function_handle_index(), FunctionHandleIndex::new(0));
        dbg!(&addmod);
        dbg!(&addmod.inline());
    }

    #[test]
    fn test_mulmod_call() {
        let module = do_compile_module(Path::new(ETH_MODULE_PATH), &[]).0;
        let addmod = MulMod::new(&module);
        // assert_eq!(addmod.function_handle_index(), FunctionHandleIndex::new(0));
        dbg!(&addmod);
        dbg!(&addmod.call());
    }
}
