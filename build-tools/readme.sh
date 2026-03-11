# Run `sh develop/cargo-readme.sh` on project root to generate `README.md` from `src/lib.rs`
# cargo install cargo-readme
cargo readme --no-badges --no-indent-headings --no-license --no-template --no-title > README.md
cd sea-orm-sync && cargo readme --no-badges --no-indent-headings --no-license --no-template --no-title > README.md