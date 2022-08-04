use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Result};
use evm::parse_program;
use lazy_static::lazy_static;
use log::log_enabled;
use log::Level;
use move_binary_format::binary_views::BinaryIndexedView;
use move_bytecode_source_map::mapping::SourceMapping;
use move_core_types::account_address::AccountAddress;
use move_core_types::value::MoveValue;
use move_disassembler::disassembler::{Disassembler, DisassemblerOptions};
use move_ir_types::location::Spanned;
use mv::function::code::intrinsic::math::u128_model::U128MathModel;
use mv::mvir::MvModule;
use regex::Regex;

pub mod clog;
pub mod color;
pub mod parse;

use parse::{SolFile, SolTest};
use test_infra::executor::{ExecutionResult, MoveExecutor};

const TEST_NAME: &str = "sol";

lazy_static! {
    pub static ref REG_PARAMS: Regex = Regex::new("[^a-z0-9]+").unwrap();
}

pub struct STest {
    bin_path: PathBuf,
    abi_path: PathBuf,
    module_name: String,
    test: SolTest,
}

impl STest {
    pub fn from_file(file: SolFile) -> Vec<STest> {
        file.tests
            .into_iter()
            .map(|test| STest {
                bin_path: file.bin_path.clone(),
                abi_path: file.abi_path.clone(),
                module_name: file.module_name.clone(),
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
            "{TEST_NAME}::{module}::{function}{sub}",
            module = &self.module_name,
            function = &self.test.func
        )
    }

    pub fn run(&self) -> Result<()> {
        let result = self.vm_run();

        let module_address = self.module_address();
        let test = &self.test;
        log::info!(
            "{wait}: {module_address}::{test:?}",
            wait = color::font_blue("WAIT")
        );

        let result = match result {
            Ok(result) => result,
            Err(err) => {
                if test.result.is_panic() {
                    return Ok(());
                } else {
                    bail!("{err:?}");
                }
            }
        };
        let result: Vec<MoveValue> = result
            .returns
            .iter()
            .map(|(actual_val, actual_tp)| {
                MoveValue::simple_deserialize(actual_val, actual_tp).unwrap()
            })
            .collect();

        if test.result.is_panic() {
            bail!("returned: {result:?}");
        }

        let expected = test.result.value().unwrap();
        if expected != &result {
            bail!("returned: {result:?}");
        }

        Ok(())
    }

    fn module_address(&self) -> String {
        format!("0x1::{}", &self.module_name)
    }

    fn vm_run(&self) -> Result<ExecutionResult> {
        let module_address = self.module_address();

        let bytecode = make_move_module(&module_address, &self.bin()?, &self.abi()?);
        let mut vm = MoveExecutor::new();
        vm.deploy("0x1", bytecode);

        let func_address = format!("{module_address}::{}", &self.test.func);
        vm.run(&func_address, &self.test.params)
    }

    fn abi(&self) -> Result<String> {
        Ok(fs::read_to_string(&self.abi_path)?)
    }

    fn bin(&self) -> Result<String> {
        Ok(fs::read_to_string(&self.bin_path)?)
    }
}

pub fn make_move_module(name: &str, eth: &str, abi: &str) -> Vec<u8> {
    let mut split = name.split("::");

    let addr = AccountAddress::from_hex_literal(split.next().unwrap()).unwrap();
    let name = split.next().unwrap();
    let program = parse_program(name, eth, abi).unwrap();
    let module = MvModule::from_evm_program(addr, U128MathModel::default(), program).unwrap();
    let compiled_module = module.make_move_module().unwrap();
    let mut bytecode = Vec::new();
    compiled_module.serialize(&mut bytecode).unwrap();

    let source_mapping = SourceMapping::new_from_view(
        BinaryIndexedView::Module(&compiled_module),
        Spanned::unsafe_no_loc(()).loc,
    )
    .unwrap();
    if log_enabled!(Level::Trace) {
        let disassembler = Disassembler::new(source_mapping, DisassemblerOptions::new());
        let dissassemble_string = disassembler.disassemble().unwrap();
        log::trace!("{}", dissassemble_string);
    }
    bytecode
}
