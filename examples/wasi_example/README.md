# SeaORM WASI + Proxy example

Simple implementation of WASI and proxy connections using Wasmtime and SeaORM.

First build the client wasm using

```bash
cargo build --target wasm32-wasi --package api
```

Then run server using

```bash
cargo run --package vm
```
