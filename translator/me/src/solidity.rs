use std::path::Path;

use anyhow::{anyhow, Result};
use eth::{compile::build_sol, Flags};
use move_core_types::account_address::AccountAddress;

use crate::{MoveExecutor, MoveExecutorInstance};

pub trait FromSolidity {
    fn from_sol<S: AsRef<Path>>(
        path: S,
        signer: AccountAddress,
        initialization_args: &str,
        flags: Flags,
    ) -> Result<MoveExecutor>;
}

impl FromSolidity for MoveExecutor {
    fn from_sol<S: AsRef<Path>>(
        path: S,
        signer: AccountAddress,
        initialization_args: &str,
        flags: Flags,
    ) -> Result<MoveExecutor> {
        let pack = build_sol(path)?;
        let cfg = translator::Config {
            contract_addr: signer,
            name: pack.name(),
            initialization_args,
            flags,
        };
        let mv = translator::translate(pack.bin_contract(), pack.abi_str(), cfg)
            .map_err(|err| anyhow!("translator: {err:?}"))?;
        let mut vm = MoveExecutor::new(pack.abi()?, flags, MoveExecutorInstance::Aptos);
        vm.deploy(&signer.to_hex_literal(), mv.bytecode)?;

        Ok(vm)
    }
}
