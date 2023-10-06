use anyhow::Result;

use wasmtime::*;
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};

lazy_static::lazy_static! {
    static ref STORE: Option<Store<WasiCtx>> = None;
}

fn query(ptr: i32, len: i32) -> i32 {
    println!("ptr: {}, len: {}", ptr, len);

    1
}

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

    linker.func_wrap("sea-orm", "query", query)?;
    linker.module(&mut store, "", &module)?;

    linker
        .get_default(&mut store, "")?
        .typed::<(), ()>(&store)?
        .call(&mut store, ())?;

    let instance = linker.instantiate(&mut store, &module)?;

    // Read a string of 19 bytes from memory at position 1048600
    // TODO - Use tokio to read asynchronously to avoid the lifetime problem of wasmtime objects
    let memory = instance.get_memory(&mut store, "memory").unwrap();
    let data = memory.data(&mut store);
    let str = std::str::from_utf8(&data[1048600..1048619]).unwrap();
    println!("str: {}", str);

    println!("Done at VM");

    Ok(())
}
