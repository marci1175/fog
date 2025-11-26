use std::process::Command;

fn main() {
    // get version
    let output = Command::new("llvm-config")
        .arg("--version")
        .output()
        .expect("failed to run llvm-config");

    let version = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=LLVM_VERSION={}", version.trim());
}