use anyhow::Result;
use bytes::Bytes;

use wasmtime::{Config, Engine};
use wit_component::ComponentEncoder;

mod runtime;
mod stream;

use {runtime::Runtime, stream::Msg};

#[async_std::main]
async fn main() -> Result<()> {
    // Transfer the wasm binary to wasm component binary

    let adapter = include_bytes!("../res/wasi_snapshot_preview1.command.wasm");

    let component = &ComponentEncoder::default()
        .module(include_bytes!(
            "../target/wasm32-wasi/debug/sea-orm-proxy-wasmtime-example-module.wasm"
        ))?
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
    let entity_base = Runtime::new(cwasm);

    use std::time::Instant;
    let now = Instant::now();

    let mut threads = Vec::new();
    for index in 0..10 {
        let mut entity = entity_base.clone();
        threads.push(std::thread::spawn(move || {
            let mut runner = entity.init().unwrap();

            let tx = runner.tx.clone();
            let rx = runner.rx.clone();

            std::thread::spawn(move || {
                runner.run().unwrap();
            });

            let data = Msg {
                id: 233,
                data: "hello".to_string(),
            };
            println!("#{index} Sending to vm: {:?}", data);
            tx.send(data).unwrap();

            let msg = rx.recv().unwrap();
            println!("#{index} Received on main: {:?}", msg);

            let data = Msg {
                id: 23333,
                data: "hi".to_string(),
            };
            println!("#{index} Sending to vm: {:?}", data);
            tx.send(data).unwrap();

            let msg = rx.recv().unwrap();
            println!("#{index} Received on main: {:?}", msg);
        }));
    }

    for thread in threads {
        thread.join().unwrap();
    }
    println!("Time elapsed: {:?}", now.elapsed());

    Ok(())
}
