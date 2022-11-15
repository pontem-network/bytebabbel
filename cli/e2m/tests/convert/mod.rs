use std::fs;
use tempfile::tempdir;

use crate::{
    add_aptos_config_for_test, aptos_init_local_profile, checking_the_file_structure, e2m,
};

#[test]
fn test_default_profile_not_found() {
    // After the test is completed, it will be deleted
    let tmp_project_folder = tempdir().unwrap();

    // Be sure to specify the profile or address for the module
    assert!(e2m(
        &["convert", "../../examples/a_plus_b.sol"],
        tmp_project_folder.as_ref(),
    )
    .is_err());
}

/// Package address
#[test]
fn test_package_address() {
    // After the test is completed, it will be deleted
    let tmp_project_folder = tempdir().unwrap();

    //  The address must start with "0x"
    assert!(e2m(
        &[
            "convert",
            "../../examples/a_plus_b.sol",
            "--profile",
            "60377c1019fdf87e372cffdcaf260e8fd7e83fe17d84b19109eaa0be597e5c5f",
        ],
        tmp_project_folder.as_ref(),
    )
    .is_err());

    e2m(
        &[
            "convert",
            "../../examples/a_plus_b.sol",
            "--profile",
            "0x42",
        ],
        tmp_project_folder.as_ref(),
    )
    .unwrap();

    checking_the_file_structure(&tmp_project_folder.as_ref().join("APlusB"), "APlusB");

    let move_toml_string =
        fs::read_to_string(tmp_project_folder.as_ref().join("APlusB/Move.toml")).unwrap();
    assert!(move_toml_string.contains(r#"self = "0x42""#));
}

/// Profile name
#[test]
fn test_profile_name() {
    // After the test is completed, it will be deleted
    let tmp_project_folder = tempdir().unwrap();
    // .aptos/config.yaml
    add_aptos_config_for_test(tmp_project_folder.as_ref()).unwrap();

    // default
    e2m(
        &["convert", "../../examples/a_plus_b.sol"],
        tmp_project_folder.as_ref(),
    )
    .unwrap();

    checking_the_file_structure(&tmp_project_folder.as_ref().join("APlusB"), "APlusB");

    let move_toml_string =
        fs::read_to_string(tmp_project_folder.as_ref().join("APlusB/Move.toml")).unwrap();
    assert!(move_toml_string.contains(
        r#"self = "0x60377c1019fdf87e372cffdcaf260e8fd7e83fe17d84b19109eaa0be597e5c5f""#
    ));

    // demo
    e2m(
        &[
            "convert",
            "../../examples/a_plus_b.sol",
            "--profile",
            "demo",
        ],
        tmp_project_folder.as_ref(),
    )
    .unwrap();

    checking_the_file_structure(&tmp_project_folder.as_ref().join("APlusB"), "APlusB");

    let move_toml_string =
        fs::read_to_string(tmp_project_folder.as_ref().join("APlusB/Move.toml")).unwrap();
    assert!(move_toml_string.contains(
        r#"self = "0x915a2a67077aeafba1003801f19630b89d8548b5cc4d91615d411cb9139cebbc""#
    ));
}

/// Directory path for saving the interface and the converted binary code
#[test]
fn test_output() {
    // After the test is completed, it will be deleted
    let tmp_project_folder = tempdir().unwrap();
    // .aptos/config.yaml
    add_aptos_config_for_test(tmp_project_folder.as_ref()).unwrap();

    // default
    let output = e2m(
        &[
            "convert",
            "../../examples/a_plus_b.sol",
            "--output",
            "package",
        ],
        tmp_project_folder.as_ref(),
    )
    .unwrap();
    println!("{output}");

    checking_the_file_structure(&tmp_project_folder.as_ref().join("package"), "APlusB");
}
