#!/bin/bash
set -e
cd sea-orm-codegen
sed -i 's/^version.*$/version = "'$1'"/' Cargo.toml
git commit -am "sea-orm-codegen $1"
cargo publish
cd ..
cd sea-orm-cli
sed -i 's/^version.*$/'"version = \"$1\"/" Cargo.toml
sed -i 's/^sea-orm-codegen [^,]*,/sea-orm-codegen = { version = "\^'$1'",/' Cargo.toml
git commit -am "sea-orm-cli $1"
cargo publish
cd ..
cd sea-orm-macros
sed -i 's/^version.*$/version = "'$1'"/' Cargo.toml
git commit -am "sea-orm-macros $1"
cargo publish
cd ..
sed -i 's/^sea-orm-macros [^,]*,/sea-orm-macros = { version = "\^'$1'",/' Cargo.toml