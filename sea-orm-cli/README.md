# SeaORM CLI

Getting Help:

```sh
cargo run -- -h
```

Running Entity Generator:

```sh
# MySQL (`--database-schema` option is ignored)
cargo run -- generate entity -u mysql://sea:sea@localhost/bakery -o out

# PostgreSQL
cargo run -- generate entity -u postgres://sea:sea@localhost/bakery -s public -o out
```
