#!/bin/bash
set -e
cd sea-orm-codegen
sed -i 's/^version.*$/version = "'$1'"/' Cargo.toml
git commit -am "sea-orm-codegen $1"
cargo publish
cd ..
sleep 30
cd sea-orm-cli
sed -i 's/^version.*$/version = "'$1'"/' Cargo.toml
sed -i 's/^sea-orm-codegen [^,]*,/sea-orm-codegen = { version = "\^'$1'",/' Cargo.toml
git commit -am "sea-orm-cli $1"
cargo publish
cd ..
sleep 30
cd sea-orm-macros
sed -i 's/^version.*$/version = "'$1'"/' Cargo.toml
git commit -am "sea-orm-macros $1"
cargo publish
cd ..
sleep 30
sed -i 's/^version.*$/version = "'$1'"/' Cargo.toml
sed -i 's/^sea-orm-macros [^,]*,/sea-orm-macros = { version = "\^'$1'",/' Cargo.toml
git commit -am "$1"
cargo publish
sleep 30
cd sea-orm-migration
sed -i 's/^version.*$/version = "'$1'"/' Cargo.toml
sed -i 's/^sea-orm-cli [^,]*,/sea-orm-cli = { version = "\^'$1'",/' Cargo.toml
sed -i 's/^sea-orm [^,]*,/sea-orm = { version = "\^'$1'",/' Cargo.toml
git commit -am "sea-orm-migration $1"
cargo publish