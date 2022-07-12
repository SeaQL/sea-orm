#!/bin/bash
set -e

# publish `sea-orm-codegen`
cd sea-orm-codegen
cargo publish
cd ..
sleep 1

# publish `sea-orm-cli`
cd sea-orm-cli
cargo publish
cd ..
sleep 1

# publish `sea-orm-macros`
cd sea-orm-macros
cargo publish
cd ..
sleep 1

# publish `sea-orm`
cargo publish
sleep 1

# publish `sea-orm-migration`
cd sea-orm-migration
cargo publish