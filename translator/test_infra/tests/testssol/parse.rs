use std::ffi::OsStr;
use std::fmt::Formatter;
use std::path::{Path, PathBuf};
use std::{fmt, fs};

use anyhow::{anyhow, bail, Result};
use serde_json::Value;

use move_core_types::value::MoveValue;

const SOL_DIRECTORY: &str = "./sol";
const BIN_DIRECTORY: &str = "./bin";

#[derive(Debug)]
pub struct SolFile {
    sol_path: PathBuf,
    pub bin_path: PathBuf,
    pub abi_path: PathBuf,
    pub module_name: String,
    pub tests: Vec<SolTest>,
}

impl SolFile {
    pub fn from_sol_dir() -> Result<Vec<SolFile>> {
        SolFile::from_dir(&PathBuf::from(SOL_DIRECTORY).canonicalize()?)
    }

    fn from_dir(sol_dir: &Path) -> Result<Vec<SolFile>> {
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
            .filter_map(|line| SolTest::try_from(line).ok())
            .collect::<Vec<SolTest>>();
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

#[derive(Clone)]
pub struct SolTest {
    pub func: String,
    pub params: String,
    pub result: SolTestResult,
}

impl TryFrom<&str> for SolTest {
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
            SolTestResult::Panic
        } else if pre_result.is_empty() {
            SolTestResult::Value(Vec::default())
        } else {
            SolTestResult::try_from(pre_result)?
        };

        Ok(SolTest {
            func: name.trim().to_string(),
            params: params.trim().to_string(),
            result,
        })
    }
}

impl fmt::Debug for SolTest {
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

#[derive(Clone)]
pub enum SolTestResult {
    Panic,
    Value(Vec<MoveValue>),
}

impl SolTestResult {
    pub fn is_panic(&self) -> bool {
        matches!(self, SolTestResult::Panic)
    }

    pub fn value(&self) -> Result<&Vec<MoveValue>> {
        match self {
            SolTestResult::Value(v) => Ok(v),
            _ => bail!("need the type SolFileTestResult::Value"),
        }
    }
}

impl fmt::Debug for SolTestResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let output = match self {
            SolTestResult::Panic => "!panic".to_string(),
            SolTestResult::Value(v) => format!("{v:?}"),
        };
        write!(f, "{output}")
    }
}

impl TryFrom<Vec<&str>> for SolTestResult {
    type Error = anyhow::Error;
    fn try_from(value: Vec<&str>) -> std::result::Result<Self, Self::Error> {
        let result = value
            .into_iter()
            .map(str_to_movevalue)
            .collect::<Result<Vec<MoveValue>>>()?;
        Ok(SolTestResult::Value(result))
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