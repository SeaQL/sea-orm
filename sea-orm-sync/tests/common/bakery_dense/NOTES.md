To regenerate entities, run at project root:
```sh
DATABASE_URL="postgres://sea:sea@localhost/bakery_chain_schema_crud_tests" cargo run --manifest-path sea-orm-cli/Cargo.toml -- generate entity -o tests/common/bakery_dense --entity-format dense --er-diagram
```