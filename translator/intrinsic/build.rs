use std::env;
use std::path::PathBuf;
use std::process::Command;

pub fn main() {
    let project_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap().as_str()).join("mv");

    Command::new("aptos")
        .stderr(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .current_dir(project_dir)
        .args(&["move", "test"])
        .output()
        .unwrap();
}
