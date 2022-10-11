use std::path::PathBuf;
use std::process::Command;

use anyhow::{ensure, Result};

const TEST_HELPER_MOVE: &str = "./resources/test_helper";

pub fn main() -> Result<()> {
    let path = PathBuf::from(TEST_HELPER_MOVE).canonicalize()?;

    // building the move test_helper project
    let r = Command::new("aptos")
        .current_dir(&path)
        .args(["move", "test"])
        // error output to the screen
        .stderr(std::process::Stdio::inherit())
        .output()?;

    ensure!(r.status.code() == Some(0), "Failed to build the project");

    let file_path = path
        .join("build")
        .join("test_helper")
        .join("bytecode_modules")
        .join("balance.mv");

    ensure!(file_path.exists(), "file not found balance.mv");

    Ok(())
}
