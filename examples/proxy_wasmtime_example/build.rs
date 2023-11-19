use std::{env, path::Path, process::Command};

fn main() {
    // Build the wasm component binary
    let pwd = Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    Command::new("cargo")
        .current_dir(pwd.clone())
        .arg("build")
        .arg("--target")
        .arg("wasm32-wasi")
        .arg("--package")
        .arg("module")
        .arg("--release")
        .status()
        .unwrap();

    println!("cargo:rerun-if-changed=module/**/*");
}
