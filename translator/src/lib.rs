use anyhow::Error;
use eth::transpile_program;
use move_core_types::account_address::AccountAddress;
use mv::translator::MvIrTranslator;
use primitive_types::U256;

pub const MAX_MEMORY: u64 = 1024 * 32;

pub fn translate(
    addr: AccountAddress,
    name: &str,
    init_args: &str,
    bytecode: &str,
    abi: &str,
) -> Result<Vec<u8>, Error> {
    let program = transpile_program(name, bytecode, init_args, abi, U256::from(addr.as_slice()))?;
    let mvir = MvIrTranslator::new(addr, MAX_MEMORY, program);
    let module = mvir.translate()?;
    let compiled_module = module.make_move_module()?;
    let mut bytecode = Vec::new();
    compiled_module.serialize(&mut bytecode)?;
    Ok(bytecode)
}
