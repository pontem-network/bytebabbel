use std::fmt::Debug;

use anyhow::{anyhow, ensure, Error, Result};
use eth::abi::call::ToCall;
use lazy_static::lazy_static;
use move_core_types::account_address::AccountAddress;
use regex::Regex;

pub mod convert;
pub mod env;
pub mod parse;

use crate::testssol::convert::ResultToString;
use crate::testssol::env::sol::EvmPack;
use env::executor::{ExecutionResult, MoveExecutor};
use eth::Flags;
use parse::{SolFile, SolTest};
use test_infra::color;
use translator::translate;

const TEST_NAME: &str = "sol";

lazy_static! {
    pub static ref REG_PARAMS: Regex = Regex::new("[^a-z0-9]+").unwrap();
}

#[derive(Debug)]
pub struct STest {
    prename: String,
    contract: EvmPack,
    test: SolTest,
}

impl STest {
    pub fn from_file(file: SolFile) -> Vec<STest> {
        file.tests
            .into_iter()
            .map(|test| STest {
                prename: file.name.clone(),
                contract: file.contract.clone(),
                test,
            })
            .collect()
    }

    pub fn from_sol_dir() -> Result<Vec<STest>> {
        let list = SolFile::from_sol_dir()?
            .into_iter()
            .filter(|file| !file.tests.is_empty())
            .flat_map(STest::from_file)
            .collect();
        Ok(list)
    }
}

impl STest {
    pub fn test_name(&self, index: usize) -> String {
        let mut sub = REG_PARAMS.replace_all(&self.test.params, "_").to_string();
        if sub.len() > 15 {
            sub = index.to_string();
        }
        if !sub.is_empty() {
            sub = format!("::{sub}");
        }
        format!(
            "{TEST_NAME}::{file}::{module}::{function}{sub}",
            file = self.prename,
            module = self.contract.name(),
            function = &self.test.func
        )
    }

    pub fn run(&self) -> Result<()> {
        let module_address = self.module_address();
        let test = &self.test;

        // move result
        let result_mv = return_val_to_string(self.run_mv());
        // sol result
        let result_evm = return_val_to_string(self.run_evm());
        log::info!(
            "{wait}: {module_address}::{test:?} {result_evm}",
            wait = color::font_blue("WAIT")
        );
        ensure!(result_evm == result_mv, "returned: {result_mv}",);

        Ok(())
    }

    pub fn run_mv(&self) -> Result<String> {
        let result = self.vm_run().map_err(|err| anyhow!("{err}"))?;
        Ok(result.returns.to_result_str())
    }

    pub fn run_evm(&self) -> Result<String> {
        use crate::testssol::env::revm::REvm;

        let abi = self.contract.abi()?;
        let ent = abi
            .by_name(&self.test.func)
            .ok_or_else(|| anyhow!("function not found in abi"))?;
        let mut callfn = ent.try_call()?;
        let tx = callfn
            .parse_and_set_inputs(&self.test.params)?
            .encode(true)?;

        let mut evm = REvm::try_from(&self.contract)?;
        evm.construct(vec![])?;
        let result_bytes = evm.run_tx(tx)?;
        log::trace!("emv result_bytes: {result_bytes:?}");

        let return_value = callfn.decode_return(result_bytes)?.to_result_str();
        log::trace!("emv result_string: {return_value:?}");

        Ok(return_value)
    }

    fn module_address(&self) -> String {
        format!("0x42::{}", &self.contract.name())
    }

    fn vm_run(&self) -> Result<ExecutionResult> {
        let module_address = self.module_address();

        let bytecode = make_move_module(
            &module_address,
            &hex::encode(self.bin()?),
            "",
            self.abi_str(),
            Flags::default(),
        )?;
        let mut vm = MoveExecutor::new(self.contract.abi()?, Flags::default());
        vm.deploy("0x42", bytecode);
        vm.run(&format!("{}::constructor", module_address), "0x42", None)
            .unwrap();

        let func_address = format!("{module_address}::{}", &self.test.func);
        vm.run(&func_address, "0x42", Some(&self.test.params))
    }

    fn abi_str(&self) -> &str {
        self.contract.abi_str()
    }

    fn bin(&self) -> Result<Vec<u8>> {
        self.contract.code()
    }
}

pub fn make_move_module(
    name: &str,
    eth: &str,
    init_args: &str,
    abi: &str,
    flags: Flags,
) -> Result<Vec<u8>, Error> {
    let mut split = name.split("::");
    let addr = AccountAddress::from_hex_literal(split.next().unwrap())?;
    let name = split.next().unwrap();
    let cfg = translator::Config {
        contract_addr: addr,
        name,
        initialization_args: init_args,
        flags,
    };
    let target = translate(eth, abi, cfg)?;
    Ok(target.bytecode)
}

fn return_val_to_string(val: Result<String>) -> String {
    match val {
        Ok(val) => val,
        Err(err) => {
            log::trace!("{err}");
            "!panic".to_string()
        }
    }
}
