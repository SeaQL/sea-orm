# SeaORM CLI

Install and Usage: 

```sh
> cargo install sea-orm-cli 
> sea-orm-cli help
```

Or: 

```sh
> cargo install --bin sea
> sea help
```

Getting Help:

```sh
cargo run -- -h
```

## Running Entity Generator:

```sh
# MySQL (`--database-schema` option is ignored)
cargo run -- generate entity -u mysql://sea:sea@localhost/bakery -o out

# SQLite (`--database-schema` option is ignored)
cargo run -- generate entity -u sqlite://bakery.db -o out

# PostgreSQL
cargo run -- generate entity -u postgres://sea:sea@localhost/bakery -s public -o out
```

## Running Migration:

- Initialize migration directory
    ```sh
    cargo run -- migrate init
    ```
- Apply all pending migrations
    ```sh
    cargo run -- migrate
    ```
    ```sh
    cargo run -- migrate up
    ```
- Apply first 10 pending migrations
    ```sh
    cargo run -- migrate up -n 10
    ```
- Rollback last applied migrations
    ```sh
    cargo run -- migrate down
    ```
- Rollback last 10 applied migrations
    ```sh
    cargo run -- migrate down -n 10
    ```
- Drop all tables from the database, then reapply all migrations
    ```sh
    cargo run -- migrate fresh
    ```
- Rollback all applied migrations, then reapply all migrations
    ```sh
    cargo run -- migrate refresh
    ```
- Rollback all applied migrations
    ```sh
    cargo run -- migrate reset
    ```
- Check the status of all migrations
    ```sh
    cargo run -- migrate status
    ```

