use anyhow::{Context, Result};
use tokio::sync::mpsc;

use wasmtime::*;
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};

lazy_static::lazy_static! {
    static ref STORE: Option<Store<WasiCtx>> = None;
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

    let (tx, mut rx) = mpsc::channel::<(usize, usize)>(32);

    let count = std::sync::Arc::new(std::sync::Mutex::new(0));
    linker.func_wrap("sea-orm", "query", move |ptr: i32, len: i32| -> i32 {
        let tx = tx.clone();
        tokio::spawn(async move {
            tx.send((ptr as usize, len as usize)).await.unwrap();
        });

        let count = count.clone();
        let mut count = count.lock().unwrap();
        *count += 1;
        *count
    })?;

    linker.module(&mut store, "", &module)?;

    linker
        .get_default(&mut store, "")?
        .typed::<(), ()>(&store)?
        .call(&mut store, ())?;

    let instance = linker.instantiate(&mut store, &module)?;

    loop {
        let (ptr, len) = rx.recv().await.context("Cannot receive message")?;
        println!("ptr: {}, len: {}", ptr, len);
        let memory = instance
            .get_memory(&mut store, "memory")
            .expect("Cannot get memory");
        let data = memory.data(&mut store);
        let str = std::str::from_utf8(&data[ptr..ptr + len]).context("Cannot convert to str")?;
        println!("str: {}", str);
    }
}
