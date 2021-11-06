# Axum with SeaORM example app

Edit `Cargo.toml` to use `sqlx-mysql` or `sqlx-postgres`.

```toml
[features]
default = ["sqlx-$DATABASE"]
```

Edit `.env` to point to your database.