# jsonrpsee with SeaORM example app

1. Modify the `DATABASE_URL` var in `.env` to point to your chosen database

1. Turn on the appropriate database feature for your chosen db in `entity/Cargo.toml` (the `"sqlx-sqlite",` line)

1. Execute `cargo run` to start the server

2. Send jsonrpc request to server

```shell
#insert
curl --location --request POST 'http://127.0.0.1:8000' \
--header 'Content-Type: application/json' \
--data-raw '{"jsonrpc": "2.0", "method": "Post.Insert", "params": [
    {
        "id":0,
        "title":"aaaaaaa",
        "text":"aaaaaaa"
    }
], "id": 2}'

#list 
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

#delete 
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

#update
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