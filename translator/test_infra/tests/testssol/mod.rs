use anyhow::{anyhow, ensure, Error, Result};
use eth::abi::call::ToCall;
use eth::bytecode::types::U256;
use eth::transpile_program;
use lazy_static::lazy_static;
use move_core_types::account_address::AccountAddress;
use move_core_types::value::MoveValue;
use mv::translator::MvIrTranslator;
use regex::Regex;
use std::fmt::Debug;

pub mod clog;
pub mod color;
pub mod convert;
pub mod env;
pub mod parse;

use crate::testssol::convert::ResultToString;
use crate::testssol::env::sol::EvmPack;
use env::executor::{ExecutionResult, MoveExecutor};
use parse::{SolFile, SolTest};

const TEST_NAME: &str = "sol";
pub const MAX_MEMORY: u64 = 1024 * 32;

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
        // log
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
        let return_value: Vec<MoveValue> = result
            .returns
            .iter()
            .map(|(actual_val, actual_tp)| {
                MoveValue::simple_deserialize(actual_val, actual_tp).unwrap()
            })
            .collect();
        Ok(return_value.to_result_str())
    }

    pub fn run_evm(&self) -> Result<String> {
        use crate::testssol::env::revm::REvm;

        let abi = self.contract.abi()?;
        let ent = abi
            .by_name(&self.test.func)
            .ok_or_else(|| anyhow!("function not found in abi"))?;
        let mut callfn = ent.try_call()?;
        let tx = callfn.parse_and_set_inputs(&self.test.params)?.encode()?;

        let evm = REvm::try_from(&self.contract)?;
        let result_bytes = evm.run_tx(tx)?;
        // @todo
        log::trace!("emv result_bytes: {result_bytes:?}");

        let return_value = callfn.decode_return(result_bytes)?.to_result_str();
        // @todo
        log::trace!("emv result_string: {return_value:?}");

        Ok(format!("{return_value}"))
    }

    fn module_address(&self) -> String {
        format!("0x1::{}", &self.contract.name())
    }

    fn vm_run(&self) -> Result<ExecutionResult> {
        let module_address = self.module_address();

        let bytecode =
            make_move_module(&module_address, &hex::encode(self.bin()?), self.abi_str())?;
        let mut vm = MoveExecutor::new();
        vm.deploy("0x1", bytecode);
        vm.run(&format!("{}::constructor", module_address), "0x1")
            .unwrap();

        let func_address = format!("{module_address}::{}", &self.test.func);
        vm.run(&func_address, &self.test.params)
    }

    fn abi_str(&self) -> &str {
        self.contract.abi_str()
    }

    fn bin(&self) -> Result<Vec<u8>> {
        self.contract.code()
    }
}

pub fn make_move_module(name: &str, eth: &str, abi: &str) -> Result<Vec<u8>, Error> {
    let mut split = name.split("::");
    let addr = AccountAddress::from_hex_literal(split.next().unwrap())?;
    let name = split.next().unwrap();
    let program = transpile_program(name, eth, abi, U256::from(addr.as_slice()))?;
    let mvir = MvIrTranslator::new(addr, program.name());
    let module = mvir.translate(MAX_MEMORY, program)?;
    let compiled_module = module.make_move_module()?;
    let mut bytecode = Vec::new();
    compiled_module.serialize(&mut bytecode)?;
    Ok(bytecode)
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
