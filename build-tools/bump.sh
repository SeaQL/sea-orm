#!/bin/bash
set -e

# In-place sed differs between GNU (Linux) and BSD (macOS) sed:
# GNU accepts `sed -i`, while BSD requires an explicit empty suffix `sed -i ''`.
# Detect once and reuse so this script works on both.
if sed --version 2>/dev/null | grep -q GNU; then
    SI=(sed -i)
else
    SI=(sed -i '')
fi

# Bump `sea-orm-codegen` version
cd sea-orm-codegen
"${SI[@]}" 's/^version.*$/version       = "'$1'"/' Cargo.toml
cd ..

# Bump `sea-orm-cli` version
cd sea-orm-cli
"${SI[@]}" 's/^version.*$/version = "'$1'"/' Cargo.toml
"${SI[@]}" 's/^sea-orm-codegen [^,]*,/sea-orm-codegen = { version = "\='$1'",/' Cargo.toml
cd ..

# Bump `sea-orm-macros` version
cd sea-orm-macros
"${SI[@]}" 's/^version.*$/version       = "'$1'"/' Cargo.toml
cd ..

# Bump `sea-orm` version
"${SI[@]}" 's/^version.*$/version       = "'$1'"/' Cargo.toml
"${SI[@]}" 's/^sea-orm-macros [^,]*,/sea-orm-macros = { version = "'~$1'",/' Cargo.toml

# Bump `sea-orm-migration` version
cd sea-orm-migration
"${SI[@]}" 's/^version.*$/version       = "'$1'"/' Cargo.toml
"${SI[@]}" 's/^sea-orm-cli [^,]*,/sea-orm-cli = { version = "'~$1'",/' Cargo.toml
"${SI[@]}" 's/^sea-orm [^,]*,/sea-orm = { version = "'~$1'",/' Cargo.toml
cd ..

# Bump `sea-orm-sync` version
cd sea-orm-sync
"${SI[@]}" 's/^version.*$/version       = "'$1'"/' Cargo.toml
cd ..

git commit -am "$1"

# Bump examples' dependency version
cd examples
find . -depth -type f -name '*.toml' -exec "${SI[@]}" 's/^version = ".*" # sea-orm version$/version = "'~$1'" # sea-orm version/' {} \;
find . -depth -type f -name '*.toml' -exec "${SI[@]}" 's/^version = ".*" # sea-orm-migration version$/version = "'~$1'" # sea-orm-migration version/' {} \;
git add .
git commit -m "update examples"
