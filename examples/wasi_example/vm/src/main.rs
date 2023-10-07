use anyhow::{Context, Result};
use tokio::sync::oneshot;

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

    let (tx_before, rx_before) = oneshot::channel::<(usize, usize)>();
    let (tx_after, rx_after) = oneshot::channel::<String>();

    let tx_before_clone = std::sync::Arc::new(std::sync::Mutex::new(Some(tx_before)));
    let rx_after_clone = std::sync::Arc::new(std::sync::Mutex::new(Some(rx_after)));
    linker.func_wrap("sea-orm", "query", move |ptr: i32, len: i32| -> i32 {
        println!("ptr: {}, len: {}", ptr, len);
        let tx_before = tx_before_clone.lock().unwrap().take().unwrap();
        println!("sending");
        tx_before
            .send((ptr as usize, len as usize))
            .expect("Cannot send message from inner");
        println!("sent");

        let rx_after = rx_after_clone.lock().unwrap().take().unwrap();
        tokio::task::block_in_place(move || {
            let ret = rx_after
                .blocking_recv()
                .context("Cannot receive message from inner")
                .unwrap();
            println!("ret: {}", ret);
        });

        1
    })?;

    linker.module(&mut store, "", &module)?;

    // TODO - Sucks, but I don't know how to do this without tokio::spawn
    tokio::spawn(async move {
        tokio::task::block_in_place(move || {
            let (ptr, len) = rx_before
                .blocking_recv()
                .context("Cannot receive message from outer")
                .unwrap();
            println!("ptr: {}, len: {}", ptr, len);
        });
    });
    linker
        .get_default(&mut store, "")?
        .typed::<(), ()>(&store)?
        .call(&mut store, ())?;

    let instance = linker.instantiate(&mut store, &module)?;

    // let memory = instance
    //     .get_memory(&mut store, "memory")
    //     .expect("Cannot get memory");
    // let data = memory.data(&mut store);
    // let str = std::str::from_utf8(&data[ptr..ptr + len]).context("Cannot convert to str")?;
    // tx_after
    //     .send(str.to_owned())
    //     .expect("Cannot send message from outer");

    println!("Done at VM");

    Ok(())
}
