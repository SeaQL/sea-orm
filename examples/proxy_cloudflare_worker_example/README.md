# SeaORM Proxy Demo for Cloudflare Workers

This is a simple Cloudflare worker written in Rust. It uses the `sea-orm` ORM to interact with SQLite that is stored in the Cloudflare D1. It also uses `axum` as the server framework.

It's inspired by the [Cloudflare Workers Demo with Rust](https://github.com/logankeenan/full-stack-rust-cloudflare-axum).

## Run

Make sure you have `npm` and `cargo` installed. Be sure to use the latest version of `nodejs` and `rust`.

```bash
npx wrangler dev
```
