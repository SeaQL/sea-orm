# SeaORM CLI Example

Prepare:

Setup a test database and configure the connection string in `.env`.
Run `bakery.sql` to setup the test table and data.

Building sea-orm-cli:

```sh
(cd ../../sea-orm-cli ; cargo build)
```

Generating entity:

```sh
../../target/debug/sea-orm-cli generate entity -o src/entity
```
