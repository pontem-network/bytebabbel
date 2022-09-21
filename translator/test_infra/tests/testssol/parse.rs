use std::ffi::OsStr;
use std::fmt::Formatter;
use std::path::{Path, PathBuf};
use std::{fmt, fs, usize};

use anyhow::{anyhow, bail, Result};
use eth::abi::entries::AbiEntries;
use eth::abi::inc_ret_param::types::ParamType;
use eth::abi::inc_ret_param::value::type_to_value::fn_params_str_split;
use rand::Rng;

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
        SolFile::from_dir(dir)
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
        let abi = self.contract.abi()?;
        self.tests = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| line.starts_with("//"))
            .map(|line| line.trim_start_matches("//").trim())
            .filter(|line| line.starts_with('#'))
            .map(|line| line.trim_start_matches('#').trim())
            .filter_map(|line| SolTest::try_from_with_fuzzing(line, &abi).ok())
            .flatten()
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
    // Ignore
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
        .split('/')
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
    fn try_from(instruction: &str) -> Result<Self> {
        let (name, part) = instruction
            .split_once('(')
            .ok_or_else(|| anyhow!("Function name and parameters not found: {}", instruction))?;
        let (params, ..) = part
            .split_once(')')
            .ok_or_else(|| anyhow!("Function parameters not found: {}", instruction))?;

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

impl SolTest {
    pub fn try_from_with_fuzzing(instruction: &str, abi: &AbiEntries) -> Result<Vec<SolTest>> {
        let ins = SolTest::try_from(instruction)?;
        let params = fn_params_str_split(&ins.params)?;
        if params.is_empty() || !params.contains(&"*") {
            return Ok(vec![ins]);
        }

        let call = abi
            .by_name(&ins.func)
            .ok_or_else(|| anyhow!("Not found fn {}", &ins.func))?;
        let inputs = call
            .inputs()
            .ok_or_else(|| anyhow!("This function has no parameters {}", &ins.func))?;

        let variants = params
            .iter()
            .zip(inputs)
            .map(|(val, inp)| {
                if val != &"*" {
                    return vec![val.to_string()];
                }

                randparam_for_tp(&inp.tp)
            })
            .collect::<Vec<Vec<_>>>();

        let mut fuzzing_params = Vec::default();
        for variation_param in variants {
            if fuzzing_params.is_empty() {
                for param in variation_param {
                    fuzzing_params.push(vec![param])
                }
                continue;
            }
            fuzzing_params = fuzzing_params
                .into_iter()
                .flat_map(|row| {
                    variation_param
                        .iter()
                        .map(|param| {
                            let mut row_new = row.clone();
                            row_new.push(param.clone());
                            row_new
                        })
                        .collect::<Vec<Vec<String>>>()
                })
                .collect();
        }

        let tests: Vec<SolTest> = fuzzing_params
            .into_iter()
            .map(|params| params.join(","))
            .map(|param| {
                let mut test = ins.clone();
                test.params = param;
                test
            })
            .collect();

        Ok(tests)
    }
}

fn randparam_for_tp(tp: &ParamType) -> Vec<String> {
    match tp {
        ParamType::Bool => {
            vec!["true".to_string(), "false".to_string()]
        }
        ParamType::Int(size) => {
            let (min, max) = match size {
                8 => (i8::MIN as isize, i8::MAX as isize),
                16 => (i16::MIN as isize, i16::MAX as isize),
                32 => (i32::MIN as isize, i32::MAX as isize),
                64 => (i64::MIN as isize, i64::MAX as isize),
                128 => (i128::MIN as isize, i128::MAX as isize),
                _ => (isize::MIN, isize::MAX),
            };

            let mut result = vec![min.to_string(), max.to_string()];
            for _ in 0..5 {
                result.push(rand::thread_rng().gen_range(min..max).to_string());
            }
            result
        }
        ParamType::UInt(size) => {
            let (min, max) = match size {
                8 => (u8::MIN as usize, u8::MAX as usize),
                16 => (u16::MIN as usize, u16::MAX as usize),
                32 => (u32::MIN as usize, u32::MAX as usize),
                64 => (u64::MIN as usize, u64::MAX as usize),
                128 => (u128::MIN as usize, u128::MAX as usize),
                _ => (usize::MIN, usize::MAX),
            };

            let mut result = vec![min.to_string(), max.to_string()];
            for _ in 0..5 {
                result.push(rand::thread_rng().gen_range(min..max).to_string());
            }
            result
        }
        ParamType::Byte(size) => {
            let mut result = Vec::default();

            let mut val = (0..*size).map(|_| u8::MIN).collect::<Vec<u8>>();
            result.push(format!("0x{}", hex::encode(&val)));

            val = (0..*size).map(|_| u8::MAX).collect::<Vec<u8>>();
            result.push(format!("0x{}", hex::encode(&val)));

            for _ in 0..5 {
                val = (0..*size)
                    .map(|_| rand::thread_rng().gen_range(u8::MIN..u8::MAX))
                    .collect::<Vec<u8>>();
                result.push(format!("0x{}", hex::encode(&val)));
            }

            result
        }
        ParamType::Address => randparam_for_tp(&ParamType::Byte(32)),
        ParamType::Bytes => {
            let mut result = Vec::default();
            result.push("0x42".to_string());

            let mut val = (0..33).map(|_| u8::MAX).collect::<Vec<u8>>();
            result.push(format!("0x{}", hex::encode(&val)));

            for _ in 0..5 {
                let size = rand::thread_rng().gen_range(0..100);
                val = (0..size)
                    .map(|_| rand::thread_rng().gen_range(u8::MIN..u8::MAX))
                    .collect::<Vec<u8>>();
                result.push(format!("0x{}", hex::encode(&val)));
            }

            result
        }
        ParamType::String => {
            let mut result = Vec::default();
            result.push("0".to_string());

            let mut val = (0..100).map(|_| "z").collect::<Vec<&str>>().join("");
            result.push(format!("0x{}", hex::encode(&val)));

            const CHARS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
            for _ in 0..5 {
                let size = rand::thread_rng().gen_range(0..100);

                val = (0..size)
                    .map(|_| CHARS[rand::thread_rng().gen_range(0..CHARS.len())])
                    .collect::<String>();
                result.push(val);
            }

            result
        }
        ParamType::Array { .. } => {
            todo!()
        }
        ParamType::Custom(_) => {
            todo!()
        }
    }
}
