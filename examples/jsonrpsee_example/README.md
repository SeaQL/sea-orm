# jsonrpsee with SeaORM example app

1. Modify the `DATABASE_URL` var in `.env` to point to your chosen database

1. Turn on the appropriate database feature for your chosen db in `api/Cargo.toml` (the `"sqlx-sqlite",` line)

1. Execute `cargo run` to start the server

1. Send jsonrpc request to server

#### Insert

```shell
curl --location --request POST 'http://127.0.0.1:8000' \
--header 'Content-Type: application/json' \
--data-raw '{"jsonrpc": "2.0", "method": "Post.Insert", "params": [
    {
        "id":0,
        "title":"aaaaaaa",
        "text":"aaaaaaa"
    }
], "id": 2}'
```

#### List

```shell
curl --location --request POST 'http://127.0.0.1:8000' \
--header 'Content-Type: application/json' \
--data-raw '{
    "jsonrpc": "2.0",
    "method": "Post.List",
    "params": [
        1,
        100
    ],
    "id": 2
}'
```

#### Delete

```shell
curl --location --request POST 'http://127.0.0.1:8000' \
--header 'Content-Type: application/json' \
--data-raw '{
    "jsonrpc": "2.0",
    "method": "Post.Delete",
    "params": [
        10
    ],
    "id": 2
}'
```

#### Update

```shell
curl --location --request POST 'http://127.0.0.1:8000' \
--header 'Content-Type: application/json' \
--data-raw '{
    "jsonrpc": "2.0",
    "method": "Post.Update",
    "params": [
        {
            "id": 1,
            "title": "1111",
            "text": "11111"
        }
    ],
    "id": 2
}'
```

Run tests:

```bash
cd api
cargo test
```

Run migration:

```bash
cargo run -p migration -- up
```

Regenerate entity:

```bash
sea-orm-cli generate entity --output-dir ./entity/src --lib --entity-format dense --with-serde both
```