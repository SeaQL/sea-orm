# SeaORM Build

A utility to generate models through a build script.

## Usage

1. Add `sea-orm-build` and `tokio` to your build dependencies in `Cargo.toml`.

```toml
[build-dependencies]
sea-orm-build = { version = "0.2", features = [
    "mysql", # or "postgres"
    "runtime-tokio-rustls", # or "runtime-tokio-native-tls"
] }
tokio = { version = "1", features = ["full"] }
```

2. Create a `build.rs` file in the root of your project.

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    sea_orm_build::generate_models("mysql://sea:sea@localhost/bakery", &["cake", "fruit"])
        .await?;
    Ok(())
}
```

3. Include the models in your app.

```rust
// src/models.rs
mod cake {
    sea_orm::include_model!("cake");
}

mod fruit {
    sea_orm::include_model!("fruit");
}
```

Queries can then be made with these models as usual.

```rust
use models::cake;

let cakes: Vec<cake::Model> = cake::Entity::find().all(db).await?;
```
