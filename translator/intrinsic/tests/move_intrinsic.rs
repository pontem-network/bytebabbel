use std::process::Command;

#[test]
pub fn test_move_intrinsic() {
    let out = Command::new("aptos")
        .stderr(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .current_dir("mv")
        .args(["move", "test"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "[ERROR] {}",
        String::from_utf8(out.stderr).unwrap()
    );
}
