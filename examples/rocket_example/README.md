![screenshot](Screenshot.png)

# Rocket with SeaORM example app

1. Modify the `url` var in `api/Rocket.toml` to point to your chosen database

1. Turn on the appropriate database feature for your chosen db in `service/Cargo.toml` (the `"sqlx-postgres",` line)

1. Execute `cargo run` to start the server

1. Visit [localhost:8000](http://localhost:8000) in browser after seeing the `🚀 Rocket has launched from http://localhost:8000` line

Run mock test on the service logic crate:

```bash
cd service
cargo test --features mock
```
