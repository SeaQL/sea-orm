#!/bin/bash
set -e

# Bump `sea-orm-codegen` version
cd sea-orm-codegen
sed -i 's/^version.*$/version = "'$1'"/' Cargo.toml
git commit -am "sea-orm-codegen $1"
cd ..
sleep 1

# Bump `sea-orm-cli` version
cd sea-orm-cli
sed -i 's/^version.*$/version = "'$1'"/' Cargo.toml
sed -i 's/^sea-orm-codegen [^,]*,/sea-orm-codegen = { version = "\^'$1'",/' Cargo.toml
git commit -am "sea-orm-cli $1"
cd ..
sleep 1

# Bump `sea-orm-macros` version
cd sea-orm-macros
sed -i 's/^version.*$/version = "'$1'"/' Cargo.toml
git commit -am "sea-orm-macros $1"
cd ..
sleep 1
sed -i 's/^version.*$/version = "'$1'"/' Cargo.toml
sed -i 's/^sea-orm-macros [^,]*,/sea-orm-macros = { version = "\^'$1'",/' Cargo.toml
git commit -am "$1" # publish sea-orm
sleep 1

# Bump `sea-orm-migration` version
cd sea-orm-migration
sed -i 's/^version.*$/version = "'$1'"/' Cargo.toml
sed -i 's/^sea-orm-cli [^,]*,/sea-orm-cli = { version = "\^'$1'",/' Cargo.toml
sed -i 's/^sea-orm [^,]*,/sea-orm = { version = "\^'$1'",/' Cargo.toml
git commit -am "sea-orm-migration $1"
cd ..
sleep 1

# Bump examples' dependency version
cd examples
find . -depth -type f -name '*.toml' -exec sed -i 's/^version = "\^.*" # sea-orm version$/version = "\^'$1'" # sea-orm version/' {} \;
find . -depth -type f -name '*.toml' -exec sed -i 's/^version = "\^.*" # sea-orm-migration version$/version = "\^'$1'" # sea-orm-migration version/' {} \;
git commit -am "update examples"