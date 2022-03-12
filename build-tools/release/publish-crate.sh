#!/usr/bin/env bash

if [ -z $CRATES_TOKEN ]
then
    echo "No crates.io token given. Doing dry-run of the publish."
    cargo publish --dry-run || echo "Publish dry-run failed"
else
    echo "Publishing crate..."
    cargo publish --token $CRATES_TOKEN
fi