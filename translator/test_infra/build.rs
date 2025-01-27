use std::path::PathBuf;
use std::process::Command;

use anyhow::{ensure, Result};

const TEST_HELPER_MOVE: &str = "./resources/mv";

pub fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=resources/mv/sources");
    println!("cargo:rerun-if-changed=resources/mv/Move.toml");

    let path = PathBuf::from(TEST_HELPER_MOVE).canonicalize()?;

    // building the move mv project
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
        .join("helper.mv");

    ensure!(file_path.exists(), "file not found helper.mv");

    Ok(())
}
