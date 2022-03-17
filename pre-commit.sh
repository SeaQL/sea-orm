#!/usr/bin/env/bash

exit_error(){
    exit_msg=$1
    if [[ "$exit_msg" != "" ]]; then
        >&2 printf "$exit_msg\n"
        exit 1
    fi
}
echo "Running pre-commit hooks for the project"
cargo test || exit_error "Cargo tests failed"
cargo clippy || exit_error "Cargo Clippy failed, lint error!"
cargo fmt --check || exit_error "Formatting failed!"
cargo build || exit_error "Build failure , please reach contact"

echo "Run cargo clean to clean your target, targets are bulky and are the compiled code of the project"