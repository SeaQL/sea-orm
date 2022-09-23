# Tonic + gRPC + SeaORM

Simple implementation of gRPC using SeaORM.

run server using

```bash
cargo run --bin server
```

run client using

```bash
cargo run --bin client
```

Run mock test on the core logic crate:

```bash
cd core
cargo test --features mock
```
