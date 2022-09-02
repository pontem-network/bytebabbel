use anyhow::{anyhow, bail, Error, Result};
use evm::abi::call::ToCall;
use evm::bytecode::types::U256;
use evm::transpile_program;
use lazy_static::lazy_static;
use move_core_types::account_address::AccountAddress;
use move_core_types::value::MoveValue;
use mv::translator::MvIrTranslator;
use regex::Regex;

pub mod clog;
pub mod color;
pub mod env;
pub mod parse;

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
        log::info!(
            "{wait}: {module_address}::{test:?}",
            wait = color::font_blue("WAIT")
        );

        // sol result
        let result_evm = self.run_evm();
        // move result
        let result_mv = self.run_mv();

        let return_value = match result_mv {
            Ok(result) => result,
            Err(err) => {
                if test.result.is_panic() {
                    return Ok(());
                } else {
                    bail!("{err:?}");
                }
            }
        };

        if test.result.is_panic() {
            bail!("returned: {return_value:?}");
        }

        let expected = test.result.value().unwrap();
        if expected != &return_value {
            bail!("returned: {return_value:?}");
        }

        Ok(())
    }

    pub fn run_mv(&self) -> Result<Vec<MoveValue>> {
        let result = self.vm_run().map_err(|err| anyhow!("{err}"))?;
        let return_value: Vec<MoveValue> = result
            .returns
            .iter()
            .map(|(actual_val, actual_tp)| {
                MoveValue::simple_deserialize(actual_val, actual_tp).unwrap()
            })
            .collect();
        Ok(return_value)
    }

    pub fn run_evm(&self) -> Result<()> {
        use crate::testssol::env::revm::REvm;

        let abi = self.contract.abi()?;
        let ent = abi
            .by_name(&self.test.func)
            .ok_or_else(|| anyhow!("function not found in abi"))?;
        let mut callfn = ent.try_call()?;
        callfn.parse_and_set_inputs(&self.test.params)?;

        dbg!(&self.test.func);
        // abi.by_name(self.test.func)
        let evm = REvm::try_from(&self.contract)?;

        todo!()
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
        //todo replace with constructor.
        vm.run(&format!("{}::init_store", module_address), "0x1")
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
