use std::ffi::OsStr;
use std::fmt::Formatter;
use std::path::{Path, PathBuf};
use std::{fmt, fs};

use anyhow::{anyhow, bail, Result};

use crate::testssol::env::sol::{build_sol_by_path, EvmPack};

const SOL_DIRECTORY: &str = "./sol";

#[derive(Debug)]
pub struct SolFile {
    sol_path: PathBuf,
    pub name: String,
    pub contract: EvmPack,
    pub tests: Vec<SolTest>,
}

impl SolFile {
    pub fn from_sol_dir() -> Result<Vec<SolFile>> {
        let dir = PathBuf::from(SOL_DIRECTORY).canonicalize()?;
        SolFile::from_dir(PathBuf::from(dir))
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

        let list: Vec<SolFile> = sol_dir
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
    // @todo
    if let Some(name) = sol_path.file_name() {
        let name = name.to_string_lossy();
        if name.starts_with("i_") || name.starts_with("l_") {
            return None;
        }
    }

    let contract = build_sol_by_path(&sol_path)
        .map_err(|err| log::error!("{err:?}"))
        .ok()?;

    let name: String = sol_path
        .to_string_lossy()
        .to_string()
        .split_once(&SOL_DIRECTORY[2..])?
        .1
        .split("/")
        .filter(|p| !p.is_empty())
        .collect::<Vec<&str>>()
        .join("::");

    Some(SolFile {
        name,
        sol_path,
        contract,
        tests: Vec::new(),
    })
}

#[derive(Clone)]
pub struct SolTest {
    pub func: String,
    pub params: String,
}

impl TryFrom<&str> for SolTest {
    type Error = anyhow::Error;
    fn try_from(instruction: &str) -> std::result::Result<Self, Self::Error> {
        let (name, part) = instruction.split_once('(').ok_or(anyhow!(
            "Function name and parameters not found: {}",
            instruction
        ))?;
        let (params, ..) = part
            .split_once(')')
            .ok_or(anyhow!("Function parameters not found: {}", instruction))?;

        Ok(SolTest {
            func: name.trim().to_string(),
            params: params.trim().to_string(),
        })
    }
}

impl fmt::Debug for SolTest {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{name}({params})",
            name = &self.func,
            params = &self.params,
        )
    }
}
