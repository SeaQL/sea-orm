To re-generate diagram, run at project root:
```sh
DATABASE_URL="postgres://sea:sea@localhost/sakila" cargo run --manifest-path sea-orm-cli/Cargo.toml -- generate entity -o tests/common/sakila --entity-format dense --er-diagram
```