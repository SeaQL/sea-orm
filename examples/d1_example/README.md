# Sea-ORM D1 Example

This example demonstrates how to use Sea-ORM with Cloudflare D1.

## Prerequisites

- [Rust](https://rustup.rs/) installed
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) installed
- [Wrangler](https://developers.cloudflare.com/workers/cli-wrangler/install-update/) CLI installed

## Setup

### 1. Create a D1 Database

Create a D1 database in your Cloudflare Workers project:

```bash
wrangler d1 create sea-orm-d1-example
```

### 2. Configure wrangler.toml

Add the D1 binding to your `wrangler.toml`:

```toml
name = "sea-orm-d1-example"
compatibility_date = "2025-01-01"

[[d1_databases]]
binding = "DB"
database_name = "sea-orm-d1-example"
database_id = "your-database-id"
```

### 3. Create the Schema

Create a `schema.sql` file:

```sql
CREATE TABLE IF NOT EXISTS cake (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL
);
```

### 4. Initialize the Database

Run the migrations:

```bash
wrangler d1 execute sea-orm-d1-example --file=./schema.sql --remote
```

## Development

### Build

```bash
wasm-pack build --target web --out-dir ./dist
```

### Deploy

```bash
wrangler deploy
```

## API Endpoints

- `GET /cakes` - List all cakes
- `POST /cakes` - Create a new cake (`{"name": "Chocolate"}`)
- `GET /cakes/:id` - Get a cake by ID
- `DELETE /cakes/:id` - Delete a cake

## Example Usage

```bash
# List all cakes
curl https://your-worker.dev/cakes

# Create a cake
curl -X POST https://your-worker.dev/cakes \
  -H "Content-Type: application/json" \
  -d '{"name": "Chocolate Cake"}'

# Get a cake
curl https://your-worker.dev/cakes/1

# Delete a cake
curl -X DELETE https://your-worker.dev/cakes/1
```

## Notes

- D1 uses SQLite-compatible SQL syntax
- D1 connections require direct access via `as_d1_connection()` because `wasm-bindgen` futures are not `Send`
- Streaming is not supported for D1; use `query_all()` instead of `stream_raw()`
