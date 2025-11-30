use std::{fs, process::Command};

fn main()
{
    // Remove temporary compiler ir debug file
    // let _ = fs::remove_file(format!("{}/input_ir.dbg", env!("CARGO_MANIFEST_DIR")));

    // get version
    let output = Command::new("llvm-config")
        .arg("--version")
        .output()
        .expect("failed to run llvm-config");

    let version = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=LLVM_VERSION={}", version.trim());
}
