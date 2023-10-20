use anyhow::Result;
use bytes::Bytes;
use std::{env, path::Path, process::Command};

use wasmtime::{Config, Engine};
use wit_component::ComponentEncoder;

mod runtime;
mod stream;

use {
    runtime::Runtime,
    stream::{RequestMsg, ResponseMsg},
};

fn main() -> Result<()> {
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

    // Transfer the wasm binary to wasm component binary
    let adapter = include_bytes!("../res/wasi_snapshot_preview1.command.wasm");
    let component = pwd.join("target/wasm32-wasi/release/module.wasm");
    let component = std::fs::read(component)?;
    let component = &ComponentEncoder::default()
        .module(&component)?
        .validate(true)
        .adapter("wasi_snapshot_preview1", adapter)?
        .encode()?;

    let mut config = Config::new();
    config.wasm_component_model(true);

    let engine = &Engine::new(&config)?;

    let cwasm = engine.precompile_component(component)?;
    let cwasm = Bytes::from(cwasm);

    // Run the prototype demo

    println!("Running prototype demo...");
    let mut runner = Runtime::new(cwasm).init()?;

    let tx = runner.tx.clone();
    let rx = runner.rx.clone();

    std::thread::spawn(move || {
        runner.run().unwrap();
    });

    loop {
        let msg = rx.recv()?;
        println!("Received on main: {:?}", msg);

        // TODO - Send the result
        loop {}
    }
}
