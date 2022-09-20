use anyhow::Error;
use eth::abi::entries::AbiEntries;
use eth::transpile_program;
pub use eth::Flags;
use intrinsic::toml_template;
use move_core_types::account_address::AccountAddress;
use mv::module::interface::move_interface;
use mv::translator::MvIrTranslator;
use primitive_types::U256;

pub const MAX_MEMORY: u64 = 1024 * 32;

#[derive(Debug)]
pub struct Config<'a> {
    pub contract_addr: AccountAddress,
    pub name: &'a str,
    pub initialization_args: &'a str,
    pub flags: Flags,
}

impl<'a> Config<'a> {
    pub fn encoded_address(&self) -> U256 {
        U256::from(self.contract_addr.as_slice())
    }
}

pub fn translate(bytecode: &str, abi: &str, config: Config) -> Result<Target, Error> {
    let abi = AbiEntries::try_from(abi)?;
    let program = transpile_program(
        config.name,
        bytecode,
        config.initialization_args,
        &abi,
        config.encoded_address(),
        config.flags,
    )?;
    let mvir = MvIrTranslator::new(config.contract_addr, MAX_MEMORY, program, config.flags);
    let module = mvir.translate()?;
    let compiled_module = module.make_move_module()?;
    let interface = move_interface(&compiled_module, &abi, config.flags)?;
    let manifest = toml_template(config.name, config.contract_addr);

    let mut bytecode = Vec::new();
    compiled_module.serialize(&mut bytecode)?;

    Ok(Target {
        bytecode,
        interface,
        manifest,
    })
}

pub struct Target {
    pub bytecode: Vec<u8>,
    pub interface: String,
    pub manifest: String,
}
