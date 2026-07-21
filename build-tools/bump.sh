#!/bin/bash
set -e

# Require taplo. Bumping the version can change the version-string length (e.g.
# `rc.N` -> stable), which shifts TOML comment alignment under `align_entries`.
# The CI Taplo job enforces that formatting, so normalize it here and fail early
# rather than produce a commit that CI will reject.
if ! command -v taplo >/dev/null 2>&1; then
    echo "error: taplo not found; install it (cargo install taplo-cli --locked) before running bump.sh" >&2
    exit 1
fi

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
"${SI[@]}" 's/^sea-orm-macros [^,]*,/sea-orm-macros = { version = "'~$1'",/' Cargo.toml
cd ..

# Normalize TOML formatting of the bumped manifests so the commit passes CI Taplo.
taplo fmt Cargo.toml sea-orm-codegen/Cargo.toml sea-orm-cli/Cargo.toml \
    sea-orm-macros/Cargo.toml sea-orm-migration/Cargo.toml sea-orm-sync/Cargo.toml

git commit -am "$1"

# Bump examples' dependency version
cd examples
find . -depth -type f -name '*.toml' -exec "${SI[@]}" 's/^version = ".*" # sea-orm version$/version = "'~$1'" # sea-orm version/' {} \;
find . -depth -type f -name '*.toml' -exec "${SI[@]}" 's/^version = ".*" # sea-orm-migration version$/version = "'~$1'" # sea-orm-migration version/' {} \;
# Re-align comments the sed above may have shifted (align_entries) so CI Taplo passes.
taplo fmt .
git add .
git commit -m "update examples"
