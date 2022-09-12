use anyhow::Error;
use eth::abi::entries::AbiEntries;
use eth::transpile_program;
use move_core_types::account_address::AccountAddress;
use mv::mv_ir::interface::move_interface;
use mv::translator::MvIrTranslator;
use primitive_types::U256;

pub const MAX_MEMORY: u64 = 1024 * 32;

pub fn translate(
    addr: AccountAddress,
    name: &str,
    init_args: &str,
    bytecode: &str,
    abi: &str,
) -> Result<Target, Error> {
    let abi = AbiEntries::try_from(abi)?;
    let program = transpile_program(name, bytecode, init_args, &abi, U256::from(addr.as_slice()))?;
    let mvir = MvIrTranslator::new(addr, MAX_MEMORY, program);
    let module = mvir.translate()?;
    let compiled_module = module.make_move_module()?;
    let interface = move_interface(&compiled_module, &abi)?;
    let mut bytecode = Vec::new();
    compiled_module.serialize(&mut bytecode)?;
    Ok(Target {
        bytecode,
        interface,
    })
}

pub struct Target {
    pub bytecode: Vec<u8>,
    pub interface: String,
}
