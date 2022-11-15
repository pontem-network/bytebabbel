use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, ensure, Result};

mod convert;

const LOCAL_CONFIG: &str = include_str!("../resources/config.yaml");

/// Run: e2m ...
pub(crate) fn e2m<P: AsRef<Path>>(args: &[&str], project_dir: P) -> Result<String> {
    run_cli(std::env!("CARGO_BIN_EXE_e2m"), args, project_dir)
}

pub(crate) fn add_aptos_config_for_test(path: &Path) -> Result<PathBuf> {
    let config_aptos_dir = path.join(".aptos");
    fs::create_dir(&config_aptos_dir)?;
    let config_file_path = config_aptos_dir.join("config.yaml");
    fs::write(&config_file_path, LOCAL_CONFIG)?;
    Ok(config_file_path)
}

fn run_cli<P: AsRef<Path>>(program: &str, args: &[&str], dir: P) -> Result<String> {
    let current_dir = dir.as_ref();
    ensure!(current_dir.exists(), "Dir {current_dir:?} does not exist",);
    ensure!(current_dir.is_dir(), "Expected directory {current_dir:?}");

    let output = std::process::Command::new(program)
        .current_dir(current_dir)
        .args(&args_relative_to_absolute(args)?)
        .output()?;

    ensure!(
        output.status.success(),
        "Command {args:?} failed with code {}. \n Error: \n{} Output: \n{}",
        output.status,
        String::from_utf8(output.stderr).unwrap_or_default(),
        String::from_utf8(output.stdout).unwrap_or_default(),
    );

    Ok(String::from_utf8(output.stdout)?)
}

#[inline]
fn args_relative_to_absolute(args: &[&str]) -> Result<Vec<String>> {
    args.iter()
        .map(|arg| {
            let arg = if arg.starts_with("./") || arg.starts_with("../") {
                PathBuf::from(arg)
                    .canonicalize()
                    .map_err(|err| anyhow!("Incorrect path {:?}. {err:?}", arg))?
                    .to_string_lossy()
                    .to_string()
            } else {
                arg.to_string()
            };
            Ok(arg)
        })
        .collect::<Result<Vec<String>>>()
}

fn checking_the_file_structure(project_dir: &Path, module_name: &str) {
    assert!(project_dir.join("Move.toml").exists());
    assert!(
        project_dir
            .join(format!("sources/{module_name}.move"))
            .exists(),
        "Expected: sources/{module_name}.move\n\
        Found: {:?}",
        project_dir.join("sources").read_dir().ok().map(|dir| {
            dir.filter_map(|path| path.ok())
                .map(|path| path.path())
                .collect::<Vec<_>>()
        })
    );
    assert!(project_dir.join(format!("{module_name}.abi")).exists());
    assert!(project_dir.join(format!("{module_name}.mv")).exists());
}
