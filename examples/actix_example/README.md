# Actix with SeaORM example app

Edit `Cargo.toml` to use `sqlx-mysql` or `sqlx-postgres`.

```toml
[features]
default = ["sqlx-$DATABASE"]
```

Edit `.env` to point to your database.

Run server with auto-reloading:

```bash
cargo install systemfd
systemfd --no-pid -s http::8000 -- cargo watch -x run
```
