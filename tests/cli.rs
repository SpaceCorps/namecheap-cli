use std::process::Command;

#[test]
fn test_login_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_namecheap-cli"))
        .arg("login")
        .arg("--help")
        .output()
        .expect("failed to execute namecheap-cli binary");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Log in and store Namecheap API credentials"));
    assert!(stdout.contains("--api-key-stdin"));
}
