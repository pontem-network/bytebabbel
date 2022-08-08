use std::ffi::OsStr;
use std::fmt::Formatter;
use std::path::{Path, PathBuf};
use std::{fmt, fs};

use anyhow::{anyhow, bail, Result};

use move_core_types::value::MoveValue;
use test_infra::sol::{build_sol, Evm};

const SOL_DIRECTORY: &str = "./sol";

#[derive(Debug)]
pub struct SolFile {
    sol_path: PathBuf,
    pub name: String,
    pub evm: Evm,
    pub tests: Vec<SolTest>,
}

impl SolFile {
    pub fn from_sol_dir() -> Result<Vec<SolFile>> {
        SolFile::from_dir(PathBuf::from(SOL_DIRECTORY))
    }

    /// The search is carried out by nested folders inclusive
    fn from_dir(sol_dir: PathBuf) -> Result<Vec<SolFile>> {
        let mut result = Vec::new();
        if path_is_sol(&sol_dir) {
            if let Some(mut solfile) = pathsol_to_solfile(sol_dir) {
                solfile.check_paths()?;
                solfile.inic_tests()?;
                result.push(solfile);
            }
            return Ok(result);
        }

        if !sol_dir.is_dir() {
            return Ok(result);
        }

        let mut list: Vec<SolFile> = sol_dir
            .read_dir()?
            .filter_map(|path| path.ok())
            .map(|item| item.path())
            .filter_map(|path| {
                SolFile::from_dir(path)
                    .map_err(|err| log::error!("{err:?}"))
                    .ok()
            })
            .flatten()
            .collect();

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

        Ok(())
    }

    fn inic_tests(&mut self) -> Result<()> {
        let content = fs::read_to_string(&self.sol_path)?;
        self.tests = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| line.starts_with("//"))
            .map(|line| line.trim_start_matches("//").trim())
            .filter(|line| line.starts_with('#'))
            .map(|line| line.trim_start_matches('#').trim())
            .filter_map(|line| SolTest::try_from(line).ok())
            .collect::<Vec<SolTest>>();
        Ok(())
    }
}

fn path_to_ext(path: &Path) -> &str {
    path.extension().and_then(OsStr::to_str).unwrap_or_default()
}

fn path_is_sol(path: &Path) -> bool {
    path.is_file() && path_to_ext(path) == "sol"
}

fn pathsol_to_solfile(sol_path: PathBuf) -> Option<SolFile> {
    let content_sol = fs::read(&sol_path).ok()?;
    let evm = build_sol(&content_sol).ok()?;

    let name: String = sol_path
        .to_string_lossy()
        .to_string()
        .split_once(SOL_DIRECTORY)?
        .1
        .split("/")
        .filter(|p| !p.is_empty())
        .collect::<Vec<&str>>()
        .join("::");

    Some(SolFile {
        name,
        sol_path,
        evm,
        tests: Vec::new(),
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
        let pre_result: Vec<&str> = part.split_whitespace().collect();

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
                .trim_end_matches('_')
                .parse::<u8>()?;
            MoveValue::U8(v)
        }
        s if s.ends_with("u64") => {
            let v = s
                .trim_end_matches("u64")
                .trim_end_matches('_')
                .parse::<u64>()?;
            MoveValue::U64(v)
        }
        s if s.ends_with("u128") => {
            let v = s
                .trim_end_matches("u128")
                .trim_end_matches('_')
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
