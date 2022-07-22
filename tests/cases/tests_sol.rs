use crate::cases::make_move_module;
use crate::common::executor::MoveExecutor;
use crate::log_init;
use anyhow::{anyhow, bail, Result};
use move_core_types::value::MoveValue;
use serde_json::Value;
use std::ffi::OsStr;
use std::fmt::Formatter;
use std::path::{Path, PathBuf};
use std::{fmt, fs};

const SOL_DIRECTORY: &str = "tests/assets/sol";
const BIN_DIRECTORY: &str = "tests/assets/bin";

#[test]
pub fn test_solc() {
    log_init();

    let sol_files =
        SolFile::from_sol_dir(&PathBuf::from(SOL_DIRECTORY).canonicalize().unwrap()).unwrap();
    let mut success = true;
    for sol in sol_files {
        if sol.tests.is_empty() {
            continue;
        }
        let module_address = format!("0x1::{}", &sol.module_name);
        let eth = fs::read_to_string(&sol.bin_path).unwrap();
        let abi = fs::read_to_string(&sol.abi_path).unwrap();
        let bytecode = make_move_module(&module_address, &eth, &abi);

        let mut vm = MoveExecutor::new();
        vm.deploy("0x1", bytecode);

        for test in sol.tests {
            let func_address = format!("{module_address}::{}", &test.func);

            let result = vm.run(&func_address, &test.params);

            print!("\r{:<15}{func_address}", "[tested]");
            let result = match result {
                Ok(result) => result,
                Err(err) => {
                    if test.result.is_panic() {
                        println!("\r{:<15}{module_address}::{test:?}", "[success]");
                        continue;
                    } else {
                        println!("\r{:<15}{module_address}::{test:?}", "[error]");
                        println!("{err:?}");
                        success = false;
                        continue;
                    }
                }
            };
            let result: Vec<MoveValue> = result
                .returns
                .iter()
                .map(|(actual_val, actual_tp)| {
                    MoveValue::simple_deserialize(&actual_val, &actual_tp).unwrap()
                })
                .collect();

            // result.returns
            if test.result.is_panic() {
                println!("\r{:<15}{module_address}::{test:?}", "[error]");
                println!("returned: {result:?}");
                success = false;
                continue;
            }

            let expected = test.result.value().unwrap();
            if expected == &result {
                println!("\r{:<15}{module_address}::{test:?}", "[success]");
            } else {
                println!("\r{:<15}{module_address}::{test:?}", "[error]");
                println!("returned: {result:?}");
                success = false;
                continue;
            }
        }
        assert!(success)
    }
    assert!(success)
}

#[derive(Debug)]
struct SolFile {
    sol_path: PathBuf,
    bin_path: PathBuf,
    abi_path: PathBuf,
    module_name: String,
    tests: Vec<SolFileTest>,
}

impl SolFile {
    pub fn from_sol_dir(sol_dir: &Path) -> Result<Vec<SolFile>> {
        let mut list: Vec<SolFile> = sol_dir
            .read_dir()?
            .filter_map(|dir| dir.ok())
            .map(|item| item.path())
            .filter(path_is_sol)
            .filter_map(pathsol_to_solfile)
            .collect();

        for s in list.iter_mut() {
            s.check_paths()?;
            s.inic_tests()?;
        }

        Ok(list)
    }
}

impl SolFile {
    fn check_paths(&self) -> Result<()> {
        if !self.sol_path.exists() {
            bail!("sol file not found: {:?}", self.sol_path);
        }
        if path_to_ext(&self.sol_path) != "sol" {
            bail!("expected extension sol -> {:?}", self.sol_path);
        }

        if !self.bin_path.exists() {
            bail!("bin file not found: {:?}", self.bin_path);
        }
        if path_to_ext(&self.bin_path) != "bin" {
            bail!("expected extension bin -> {:?}", self.bin_path);
        }

        if !self.abi_path.exists() {
            bail!("abi file not found: {:?}", self.abi_path);
        }
        if path_to_ext(&self.abi_path) != "abi" {
            bail!("expected extension abi -> {:?}", self.abi_path);
        }

        Ok(())
    }

    fn inic_tests(&mut self) -> Result<()> {
        let content = fs::read_to_string(&self.sol_path)?;
        self.tests = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| line.starts_with("//"))
            .map(|line| line.trim_start_matches("//").trim())
            .filter(|line| line.starts_with("#"))
            .map(|line| line.trim_start_matches("#").trim())
            .filter_map(|line| SolFileTest::try_from(line).ok())
            .collect::<Vec<SolFileTest>>();
        Ok(())
    }
}

fn path_to_ext(path: &Path) -> &str {
    path.extension().and_then(OsStr::to_str).unwrap_or_default()
}

fn path_is_sol(path: &PathBuf) -> bool {
    path.is_file() && path_to_ext(path) == "sol"
}

fn pathsol_to_solfile(sol_path: PathBuf) -> Option<SolFile> {
    let file_name = sol_path.file_name()?.to_string_lossy().to_string();
    let ast_path = PathBuf::from(format!("{BIN_DIRECTORY}/{}_json.ast", file_name))
        .canonicalize()
        .ok()?;

    let content_ast = fs::read_to_string(&ast_path).ok()?;
    let ast_json: Value = serde_json::from_str(&content_ast).ok()?;
    let module_name = ast_json
        .get("exportedSymbols")
        .and_then(|ojg| ojg.as_object())
        .and_then(|m| m.iter().next().map(|(name, _)| name.clone()))?;

    let bin_path = PathBuf::from(format!("{BIN_DIRECTORY}/{}.bin", &module_name))
        .canonicalize()
        .ok()?;
    let abi_path = PathBuf::from(format!("{BIN_DIRECTORY}/{}.abi", &module_name))
        .canonicalize()
        .ok()?;

    Some(SolFile {
        sol_path,
        bin_path,
        abi_path,
        module_name,
        tests: Vec::default(),
    })
}

struct SolFileTest {
    func: String,
    params: String,
    result: SolFileTestResult,
}

impl TryFrom<&str> for SolFileTest {
    type Error = anyhow::Error;
    fn try_from(instruction: &str) -> std::result::Result<Self, Self::Error> {
        let (name, part) = instruction.split_once('(').ok_or(anyhow!(
            "Function name and parameters not found: {}",
            instruction
        ))?;
        let (params, part) = part
            .split_once(')')
            .ok_or(anyhow!("Function parameters not found: {}", instruction))?;
        let pre_result: Vec<&str> = part.trim().split_whitespace().collect();

        let result = if pre_result.contains(&"!panic") {
            SolFileTestResult::Panic
        } else if pre_result.is_empty() {
            SolFileTestResult::Value(Vec::default())
        } else {
            SolFileTestResult::try_from(pre_result)?
        };

        Ok(SolFileTest {
            func: name.trim().to_string(),
            params: params.trim().to_string(),
            result,
        })
    }
}

impl fmt::Debug for SolFileTest {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{name}({params}) {result:?}",
            name = &self.func,
            params = &self.params,
            result = &self.result
        )
    }
}

enum SolFileTestResult {
    Panic,
    Value(Vec<MoveValue>),
}

impl fmt::Debug for SolFileTestResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let output = match self {
            SolFileTestResult::Panic => "!panic".to_string(),
            SolFileTestResult::Value(v) => format!("{v:?}"),
        };
        write!(f, "{output}")
    }
}

impl SolFileTestResult {
    pub fn is_panic(&self) -> bool {
        matches!(self, SolFileTestResult::Panic)
    }

    pub fn value(&self) -> Result<&Vec<MoveValue>> {
        match self {
            SolFileTestResult::Value(v) => Ok(v),
            _ => bail!("need the type SolFileTestResult::Value"),
        }
    }
}

impl TryFrom<Vec<&str>> for SolFileTestResult {
    type Error = anyhow::Error;
    fn try_from(value: Vec<&str>) -> std::result::Result<Self, Self::Error> {
        let result = value
            .into_iter()
            .map(str_to_movevalue)
            .collect::<Result<Vec<MoveValue>>>()?;
        Ok(SolFileTestResult::Value(result))
    }
}

fn str_to_movevalue(value: &str) -> Result<MoveValue> {
    let result = match value {
        "true" => MoveValue::Bool(true),
        "false" => MoveValue::Bool(false),
        s if s.ends_with("u8") => {
            let v = s
                .trim_end_matches("u8")
                .trim_end_matches("_")
                .parse::<u8>()?;
            MoveValue::U8(v)
        }
        s if s.ends_with("u64") => {
            let v = s
                .trim_end_matches("u64")
                .trim_end_matches("_")
                .parse::<u64>()?;
            MoveValue::U64(v)
        }
        s if s.ends_with("u128") => {
            let v = s
                .trim_end_matches("u128")
                .trim_end_matches("_")
                .parse::<u128>()?;
            MoveValue::U128(v)
        }
        s if s.parse::<u128>().is_ok() => {
            let v = s.parse::<u128>()?;
            MoveValue::U128(v)
        }
        _ => bail!("Unknown type: {value}"),
    };
    Ok(result)
}
