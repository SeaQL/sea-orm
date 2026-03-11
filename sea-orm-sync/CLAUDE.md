# sea-orm-sync

Do **not** edit code in `sea-orm-sync/src/` or `sea-orm-sync/tests/` directly. All source and test files are generated from the main `src/` and `tests/` directories by running:

```bash
cd /path/to/sea-orm
bash build-tools/make-sync.sh
```

The script copies `src/` and `tests/` into `sea-orm-sync/`, then applies sed transforms to strip async/await, remove `#[async_trait]`, replace `futures_util::lock::Mutex` with `std::sync::Mutex`, etc.

## Workflow

1. Make all code changes in the **root** `src/` and `tests/` directories.
2. Edit `sea-orm-sync/Cargo.toml` directly if dependency or feature changes are needed.
3. Regenerate sync code: `bash build-tools/make-sync.sh`
4. Build and test: `cd sea-orm-sync && cargo check --features rusqlite`

## Gotchas

- **`make-sync.sh` requires full filesystem access.** It uses `sed -i`, `find`, `cp -r`, and `cargo fmt`. Running it inside a sandboxed environment (e.g. Cursor's default sandbox) will silently produce broken output — the sed substitutions won't apply, leaving `async_trait`, `futures_util`, `.await`, etc. in the generated code. Always run with full permissions.
- **The root crate cannot compile with `--features rusqlite`** because it depends on `sea_query_rusqlite`, which is only wired up in `sea-orm-sync/Cargo.toml`. To check rusqlite code, you must regenerate and build from `sea-orm-sync/`.
- **`#[sea_orm_macros::test]`** gates tests on `feature = "rusqlite"` in the sync crate (and on `sqlx-*` features in the async crate). When running tests, pass `--features rusqlite` and set `DATABASE_URL`, e.g.: `DATABASE_URL="sqlite::memory:" cargo test --features rusqlite --test transaction_tests`
