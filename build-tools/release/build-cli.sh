#!/usr/bin/env bash
set -ex

if [[ -z "$GITHUB_REF" ]]
then
  echo "GITHUB_REF must be set"
  exit 1
fi
TAG=${GITHUB_REF#*/tags/}
host=$(rustc -Vv | grep ^host: | sed -e "s/host: //g")

root=$(pwd)

# Build
cd sea-orm-cli
cargo build --bin sea-orm-cli --release
cli_name="sea-orm-cli-$TAG-$host"

# Package
cd target/release
case $1 in 
    macos*)
        # There is a bug with BSD tar on macOS where the first 8MB of the file are
        # sometimes all NUL bytes. See https://github.com/actions/cache/issues/403
        # and https://github.com/rust-lang/cargo/issues/8603 for some more
        # information. An alternative solution here is to install GNU tar, but
        # flushing the disk cache seems to work, too.
        sudo /usr/sbin/purge
        cli_archive=$root/$cli_name.tar.gz
        # Running strip with Cargo.toml requires Rust 1.59.0, which is higher than the MSRV
        strip ./sea-orm-cli
        tar --create --file $cli_archive ./sea-orm-cli
        ;;
    ubuntu*)
        cli_archive=$root/$cli_name.tar.gz
        # Running strip with Cargo.toml requires Rust 1.59.0, which is higher than the MSRV
        strip ./sea-orm-cli
        tar --create --file $cli_archive ./sea-orm-cli
        ;;
    windows*)
        # TODO: Figure out how to strip windows binaries
        cli_archive=$root/$cli_name.zip
        7z a  $cli_archive ./sea-orm-cli.exe
        ;;
    *)
        echo "OS not specified as first argument"
        exit 2
        ;;
esac
cd $root

# Upload
if [[ -z "$GITHUB_TOKEN" ]]
then
  echo "$GITHUB_TOKEN not set, skipping deploy."
else
  hub release edit -m "" --attach $cli_archive $TAG
fi