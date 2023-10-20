use std::env;
use std::process::Command;

fn main() {
    let pwd = env::current_dir().unwrap();

    Command::new("cargo")
        .current_dir(pwd.join("module"))
        .arg("build")
        .arg("--target")
        .arg("wasm32-wasi")
        .arg("--package")
        .arg("sea-orm-proxy-wasmtime-example-module")
        .arg("--release")
        .status()
        .unwrap();
}
