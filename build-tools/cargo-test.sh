#!/bin/bash
set -e

find $1 -type f -name 'Cargo.toml' -print0 | xargs -t -0 -I {} cargo update --manifest-path {}
find $1 -type f -name 'Cargo.toml' -print0 | xargs -t -0 -I {} cargo build --manifest-path {}
find $1 -type f -name 'Cargo.toml' -print0 | xargs -t -0 -I {} cargo test --manifest-path {}
! [ -d "$1/service" ] || find $1/service -type f -name 'Cargo.toml' -print0 | xargs -t -0 -I {} cargo test --manifest-path {} --features mock