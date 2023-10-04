use anyhow::Result;

use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    let engine = Engine::default();
    let module = Module::from_file(&engine, "./target/wasm32-wasi/debug/api.wasm")?;

    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()?
        .build();

    let mut linker = Linker::new(&engine);
    let mut store = Store::new(&engine, wasi);
    wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

    linker.func_wrap("sea-orm", "query", |ptr: i32, len: i32| -> i32 {
        println!("ptr: {}, len: {}", ptr, len);
        1
    })?;
    linker.module(&mut store, "", &module)?;

    linker
        .get_default(&mut store, "")?
        .typed::<(), ()>(&store)?
        .call(&mut store, ())?;

    println!("Done at vm");
    Ok(())
}
