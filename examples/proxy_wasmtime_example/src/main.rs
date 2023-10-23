use anyhow::Result;
use bytes::Bytes;

use sea_orm::{ConnectionTrait, Database, DatabaseBackend, ProxyExecResult, Statement};
use wasmtime::{Config, Engine};
use wit_component::ComponentEncoder;

mod runtime;
mod stream;

use {
    runtime::Runtime,
    stream::{RequestMsg, ResponseMsg},
};

#[async_std::main]
async fn main() -> Result<()> {
    // Transfer the wasm binary to wasm component binary
    let adapter = include_bytes!("../res/wasi_snapshot_preview1.command.wasm");
    let pwd = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf();
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

    // Create the database connection
    println!("Creating database connection...");
    let db = Database::connect("sqlite::memory:").await?;
    db.execute(Statement::from_string(
        DatabaseBackend::Sqlite,
        r#"
            CREATE TABLE IF NOT EXISTS posts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                text TEXT NOT NULL
            )
        "#,
    ))
    .await?;

    // Run the prototype demo
    println!("Running prototype demo...");
    let mut runner = Runtime::new(cwasm).init()?;

    let tx = runner.tx.clone();
    let rx = runner.rx.clone();

    std::thread::spawn(move || {
        runner.run().unwrap();
    });

    while let Ok(msg) = rx.recv() {
        match msg {
            RequestMsg::Execute(sql) => {
                let ret: ProxyExecResult = db
                    .execute(Statement::from_string(DatabaseBackend::Sqlite, sql))
                    .await?
                    .into();
                println!("Execute result: {:?}", ret);
                let ret = ResponseMsg::Execute(ret);
                tx.send(ret)?;
            }
            RequestMsg::Query(sql) => {
                let ret: Vec<serde_json::Value> = db
                    .query_all(Statement::from_string(DatabaseBackend::Sqlite, sql))
                    .await?
                    .iter()
                    .map(|r| sea_orm::from_query_result_to_proxy_row(&r))
                    .map(|r| {
                        // This demo only converts it to json value currently.
                        // But it can be converted to any other format that includes the type information.
                        // You can use 'match' to deal the type of the value on sea_orm::Value.
                        r.into()
                    })
                    .collect();
                println!("Query result: {:?}", ret);

                let ret = ResponseMsg::Query(ret);
                tx.send(ret)?;
            }
            RequestMsg::Debug(msg) => {
                println!("VM Debug: {}", msg);
            }
        }
    }

    Ok(())
}
