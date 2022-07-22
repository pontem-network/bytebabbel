#![feature(main_separator_str)]
extern crate wax;

use std::env;
use std::path::PathBuf;
use std::path::MAIN_SEPARATOR_STR;
use std::process::{exit, Command};

const BUILD_CONTRACTS_SH: &str = "build.sh";
const BUILD_CONTRACTS_DIR: &str = "bin";
const SOURCE_CONTRACTS_DIR: &str = "sol";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    debug_info_print_env();

    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let source_path = root.join(SOURCE_CONTRACTS_DIR);
    let script_path = root.join(BUILD_CONTRACTS_SH);
    let output_path = root.join(BUILD_CONTRACTS_DIR);

    let result = Command::new("bash")
        .arg(&script_path)
        .current_dir(&root)
        .envs(env::vars())
        .output()?;

    if !result.status.success() {
        let error = String::from_utf8(result.stderr).unwrap();
        let status = result.status.code().unwrap_or_default();
        println!("cargo:warning=test resources build faild with: {status}");
        println!("{error}");
        exit(1);
    }

    let output = String::from_utf8(result.stdout).unwrap();
    println!("{output}");

    // list all files to watch:
    {
        use wax::{Glob, LinkBehavior};

        let source_path = source_path.display().to_string();
        let glob = Glob::new("**/*.{sol,bin,evm,sol_json.ast}")
            .map_err(map_err_warn)
            .unwrap_or_else(|_| unreachable!("sol files not found"));
        let walker = glob
            .walk_with_behavior(&source_path, LinkBehavior::ReadTarget)
            .not(["**/.*/**"])
            .map_err(map_err_warn)
            .expect("invalid pattern");
        for entry in walker {
            let entry = entry.map_err(map_err_warn)?;
            println!("cargo:rerun-if-changed={}", entry.path().display());

            // propagate to crate cfg:
            let path = entry.path();
            if path
                .display()
                .to_string()
                .contains(&output_path.display().to_string())
            {
                let path = PathBuf::from(
                    &path.display().to_string()
                        [(root.display().to_string().len() + MAIN_SEPARATOR_STR.len())..],
                );
                let stem = path
                    .file_stem()
                    .expect("file name")
                    .to_string_lossy()
                    .replace('.', "_")
                    .replace(' ', "_");
                println!("cargo:rustc-env={}={}", stem, path.display());
                debug_println(format!("+ENV: {}={}", stem, path.display()));
            }
        }
    }

    Ok(())
}

fn map_err_warn<Err: std::fmt::Display>(err: Err) -> Err {
    println!("cargo:warning=error: {err}");
    err
}

fn debug_info_print_env() {
    match std::env::var("PROFILE") {
        Ok(s) if s == "DEBUG" => { /* continue */ }
        _ => return,
    }

    for (k, v) in env::vars() {
        println!("ENV: {k}: {v}")
    }
}

fn debug_println<S: AsRef<str>>(s: S) {
    let message = s.as_ref();
    match std::env::var("PROFILE") {
        Ok(s) if s == "DEBUG" => {
            println!("{message}")
        }
        _ => {}
    }
}