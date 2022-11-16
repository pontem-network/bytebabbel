use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, ensure, Result};
use move_core_types::account_address::AccountAddress;
use test_infra::color::{font_green, font_yellow};

mod convert;

const APTOS_CONFIG_FOR_TEST: &str = include_str!("../resources/config.yaml");

/// Run: e2m ...
pub(crate) fn e2m<P: AsRef<Path>>(args: &[&str], project_dir: P) -> Result<String> {
    run_cli(std::env!("CARGO_BIN_EXE_e2m"), args, project_dir)
}

/// Run: aptos init
///     --profile <PROFILE_NAME>
///     --rest-url http://localhost:8080
///     --faucet-url http://localhost:8081
///     --network local
///     --assume-yes
pub(crate) fn aptos_init_local_profile<P: AsRef<Path>>(
    dir: P,
    profile_name: &str,
) -> Result<String> {
    run_cli(
        "aptos",
        &[
            "init",
            "--profile",
            profile_name,
            "--rest-url",
            "http://localhost:8080",
            "--faucet-url",
            "http://localhost:8081",
            "--network",
            "local",
            "--assume-yes",
        ],
        dir,
    )
}

/// $ aptos account fund-with-faucet --account <...> --amount 10000000 --profile <...>
pub(crate) fn aptos_add_coins<P: AsRef<Path>>(dir: P, profile_name: &str) -> Result<String> {
    run_cli(
        "aptos",
        &[
            "account",
            "fund-with-faucet",
            "--account",
            profile_name,
            "--amount",
            "50000000",
            "--profile",
            profile_name,
        ],
        dir,
    )
}

/// aptos move publish \
///   --max-gas 10000 \
///   --assume-yes
pub(crate) fn aptos_publish_package<P: AsRef<Path>>(move_package_dir: P) -> Result<String> {
    run_cli(
        "aptos",
        &["move", "publish", "--max-gas", "10000", "--assume-yes"],
        move_package_dir,
    )
}

/// add file: .aptos/config.yaml
pub(crate) fn add_aptos_config_for_test(path: &Path) -> Result<PathBuf> {
    let config_aptos_dir = path.join(".aptos");
    fs::create_dir(&config_aptos_dir)?;
    let config_file_path = config_aptos_dir.join("config.yaml");
    fs::write(&config_file_path, APTOS_CONFIG_FOR_TEST)?;
    Ok(config_file_path)
}

fn aptos_profile_name_and_account_address(
    root_dir: &Path,
) -> Option<BTreeMap<String, AccountAddress>> {
    let configs: serde_yaml::Value =
        serde_yaml::from_str(&fs::read_to_string(root_dir.join(".aptos/config.yaml")).ok()?)
            .ok()?;
    let profiles = configs
        .get("profiles")?
        .as_mapping()?
        .into_iter()
        .filter_map(|(profile_name, configs)| {
            let name = profile_name.as_str()?.to_string();
            let address = AccountAddress::from_hex(configs.get("account")?.as_str()?).ok()?;
            Some((name, address))
        })
        .collect();
    Some(profiles)
}

fn run_cli<P: AsRef<Path>>(program: &str, args: &[&str], current_dir: P) -> Result<String> {
    let current_dir = current_dir.as_ref();
    ensure!(current_dir.exists(), "Dir {current_dir:?} does not exist",);
    ensure!(current_dir.is_dir(), "Expected directory {current_dir:?}");

    let args = args_relative_to_absolute(args)?;

    println!(
        "{} {}",
        font_green(&format!("$ {}", program_name(program))),
        args.join(" ")
    );
    let output = std::process::Command::new(program)
        .current_dir(current_dir)
        .args(&args)
        .output()?;

    ensure!(
        output.status.success(),
        "Command {args:?} failed with code {}. \n Error: \n{} Output: \n{}",
        output.status,
        String::from_utf8(output.stderr).unwrap_or_default(),
        String::from_utf8(output.stdout).unwrap_or_default(),
    );
    let output = String::from_utf8(output.stdout)?;
    println!("{}: {output}", font_yellow("Output"));

    Ok(output)
}

#[inline]
fn program_name(program: &str) -> String {
    program
        .split('/')
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
        .last()
        .unwrap_or_default()
        .to_string()
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
