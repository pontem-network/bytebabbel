use crate::e2m;
use tempfile::tempdir;

#[test]
fn test_converter() {
    // After the test is completed, it will be deleted
    let tmp_project_folder = tempdir().unwrap();

    // Be sure to specify the profile or address for the module
    assert!(e2m(
        &["convert", "./examples/a_plus_b.sol"],
        tmp_project_folder.as_ref(),
    )
    .is_err());

    let output = e2m(
        &[
            "convert",
            "/home/vmm/projects/pontem-network/eth2move/examples/a_plus_b.sol",
            "--profile",
            "0x42",
        ],
        tmp_project_folder.as_ref(),
    )
    .unwrap();

    dbg!(output);

    todo!()
}
