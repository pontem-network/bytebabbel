use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

pub fn main() {
    println!("cargo:rerun-if-changed=mv/sources/template.move");
    let project_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap().as_str()).join("mv");

    Command::new("aptos")
        .stderr(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .current_dir(&project_dir)
        .args(&["move", "compile"])
        .output()
        .unwrap();

    let module = project_dir
        .join("build")
        .join("intrinsic")
        .join("bytecode_modules")
        .join("template.mv");

    fs::copy(module, project_dir.join("build").join("template.mv")).unwrap();
}
