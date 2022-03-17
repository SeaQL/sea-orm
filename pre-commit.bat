
echo "Running pre-commit hooks for the project"

echo "\n Cargo will be testing now"
cargo test 

echo "\n Cargo will be checking Clippy now"
cargo clippy 

echo "\n Cargo will be checking for formatting now"
cargo fmt --check

echo "\n Cargo will be building"
cargo build 

echo "Run cargo clean to clean your target, targets are bulky and are the compiled code of the project"