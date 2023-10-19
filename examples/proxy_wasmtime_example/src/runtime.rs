use anyhow::{anyhow, Result};
use bytes::Bytes;
use flume::{Receiver, Sender};

use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store,
};
use wasmtime_wasi::preview2::{
    command::{self, sync::Command},
    Table, WasiCtx, WasiCtxBuilder, WasiView,
};

use crate::stream::{InputStream, Msg, OutputStream};

struct Ctx {
    wasi: WasiCtx,
    table: Table,
}

impl WasiView for Ctx {
    fn ctx(&self) -> &WasiCtx {
        &self.wasi
    }
    fn ctx_mut(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
    fn table(&self) -> &Table {
        &self.table
    }
    fn table_mut(&mut self) -> &mut Table {
        &mut self.table
    }
}

#[derive(Clone)]
pub struct Runtime {
    engine: Engine,
    component: Component,
}

pub struct Runner {
    store: Store<Ctx>,
    component: Component,
    linker: Linker<Ctx>,

    pub tx: Sender<Msg>,
    pub rx: Receiver<Msg>,
}

impl Runtime {
    pub fn new(bin: Bytes) -> Self {
        let mut config = Config::new();
        config.wasm_component_model(true);
        let engine = Engine::new(&config).unwrap();

        let component = unsafe { Component::deserialize(&engine, &bin).unwrap() };

        Self { engine, component }
    }

    pub fn init(&mut self) -> Result<Runner> {
        let mut linker = Linker::new(&self.engine);
        command::sync::add_to_linker(&mut linker).unwrap();

        let mut table = Table::new();
        let mut wasi = WasiCtxBuilder::new();
        wasi.inherit_stderr();

        let (tx_in, rx_in) = flume::unbounded();
        let (tx_out, rx_out) = flume::unbounded();

        let input_stream = InputStream {
            tasks: Default::default(),
        };
        let output_stream = OutputStream { tx: tx_out };

        let rx = rx_in.clone();
        let tasks = input_stream.tasks.clone();
        std::thread::spawn(move || {
            while let Ok(msg) = rx.recv() {
                tasks.lock().unwrap().push(msg);
            }
        });

        wasi.stdin(input_stream, wasmtime_wasi::preview2::IsATTY::No);
        wasi.stdout(output_stream, wasmtime_wasi::preview2::IsATTY::No);

        let wasi = wasi.build(&mut table).unwrap();
        let store = Store::new(&self.engine, Ctx { wasi, table });

        Ok(Runner {
            store,
            component: self.component.clone(),
            linker,

            tx: tx_in,
            rx: rx_out,
        })
    }
}

impl Runner {
    pub fn run(&mut self) -> Result<()> {
        let (command, _) = Command::instantiate(&mut self.store, &self.component, &self.linker)?;

        command
            .wasi_cli_run()
            .call_run(&mut self.store)?
            .map_err(|()| anyhow!("guest command returned error"))?;

        Ok(())
    }
}
