use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{ensure, Result};
use tempfile::tempdir;

mod convert;

/// Run: e2m ...
pub(crate) fn e2m<P: AsRef<Path>>(args: &[&str], project_dir: P) -> Result<String> {
    run_cli(std::env!("CARGO_BIN_EXE_e2m"), &args, project_dir)
}

/// Run: aptos init --profile <PROFILE_NAME>
///     --rest-url http://localhost:8080
///     --faucet-url http://localhost:8081
pub(crate) fn aptos_init_local_profile<P: AsRef<Path>>(
    dir: P,
    profile_name: &str,
) -> Result<String> {
    let args = [
        "init",
        "--profile",
        profile_name,
        " --network",
        "local",
        "--rest-url",
        "http://localhost:8080",
        "--faucet-url",
        "http://localhost:8081",
    ];
    run_cli("aptos", &args, dir)
}

fn run_cli<P: AsRef<Path>>(program: &str, args: &[&str], dir: P) -> Result<String> {
    let current_dir = dir.as_ref();
    ensure!(current_dir.exists(), "Dir {current_dir:?} does not exist",);
    ensure!(current_dir.is_dir(), "Expected directory {current_dir:?}");

    dbg!(program);
    dbg!(args);
    dbg!(current_dir);

    let output = std::process::Command::new(program)
        .current_dir(current_dir)
        .args(args)
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
