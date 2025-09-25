# Bakery Schema

## MySQL

Assume the database is named `bakery`:

```sql
CREATE DATABASE bakery;
GRANT ALL PRIVILEGES ON bakery.* TO sea;
```

## SQLite

```sh
export DATABASE_URL=sqlite://../bakery.db?mode=rwc
```

# Re-generate entities

```sh
sea-orm-cli generate entity --output-dir src/entity
```

# Running Migrator CLI

- Apply all pending migrations
    ```sh
    cargo run
    ```
    ```sh
    cargo run -- up
    ```
- Apply first 10 pending migrations
    ```sh
    cargo run -- up -n 10
    ```
- Rollback last applied migrations
    ```sh
    cargo run -- down
    ```
- Rollback last 10 applied migrations
    ```sh
    cargo run -- down -n 10
    ```
- Drop all tables from the database, then reapply all migrations
    ```sh
    cargo run -- fresh
    ```
- Rollback all applied migrations, then reapply all migrations
    ```sh
    cargo run -- refresh
    ```
- Rollback all applied migrations
    ```sh
    cargo run -- reset
    ```
- Check the status of all migrations
    ```sh
    cargo run -- status
    ```
