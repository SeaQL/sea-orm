## Use a local SeaORM version

Add this to your `Cargo.toml` for you local project to test out your changes:

```toml
[patch.crates-io]
sea-orm = { path = "../sea-orm" }
```

## Before submitting PR

### Run `clippy` and `fmt`

We need nightly to do the formatting,

```sh
cargo +nightly fmt
```

If you don't have nightly then at least run:

```sh
cargo fmt && git checkout -- ./sea-orm-codegen/src/tests_cfg/
```

We use latest stable clippy:

```sh
cargo clippy --all
```

### Running unit tests

Just do:

```sh
cargo test --lib
cargo test --doc
```

### Launch some databases

There is a docker compose under `build-tools`, but usually I just pick the ones I need from `build-tools/docker-crete.sh`:

```sh
docker run \
    --name "postgres-14" \
    --env POSTGRES_USER="sea" \
    --env POSTGRES_PASSWORD="sea" \
    -d -p 5432:5432 postgres:14
```

### Running integration tests

You need to supply the right feature flags to run integration tests:

```sh
DATABASE_URL="sqlite::memory:"              cargo test --features sqlx-sqlite,runtime-tokio              --test crud_tests
DATABASE_URL="mysql://sea:sea@localhost"    cargo test --features sqlx-mysql,runtime-tokio-native-tls    --test crud_tests
DATABASE_URL="postgres://sea:sea@localhost" cargo test --features sqlx-postgres,runtime-tokio-native-tls --test crud_tests
```

Or use `--tests` to run all tests, which can take a while.
