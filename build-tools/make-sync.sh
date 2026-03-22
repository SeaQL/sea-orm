#!/bin/bash
set -euo pipefail

if sed --version 2>/dev/null | grep -q "GNU" ; then
    SED_INPLACE=(-i)
else
    SED_INPLACE=(-i '')
fi

sed_in_place() {
    local expression="$1"
    shift
    sed "${SED_INPLACE[@]}" "$expression" "$@"
}

replace_rs() {
    local expression="$1"
    shift
    find "$@" -type f -name '*.rs' -exec sed "${SED_INPLACE[@]}" "$expression" {} +
}

rm -rf sea-orm-sync/src
rm -rf sea-orm-sync/tests
cp -r src sea-orm-sync
cp -r tests sea-orm-sync
cp examples/quickstart/src/main.rs sea-orm-sync/examples/quickstart/src/main.rs
rm -rf sea-orm-sync/src/bin
cd sea-orm-sync

replace_rs "s/Pin<Box<dyn Future<Output = Result<Self::Stream<'a>, DbErr>> + 'a + Send>>/Result<Self::Stream<'a>, DbErr>/" src
replace_rs "s/Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'b>>/Result<T, E>/" src
replace_rs "s/Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>/Result<T, E>/" src
replace_rs 's/Box::pin(async move {/({/' src
replace_rs 's/Box::pin(async move {/({/' tests
replace_rs 's/async //' src
replace_rs 's/async //' tests
replace_rs 's/async //' examples
replace_rs 's/\.await//' src
replace_rs 's/\.await//' tests
replace_rs 's/\.await//' examples
replace_rs '/#\[async_trait::async_trait\]/d' src
replace_rs '/#\[async_trait::async_trait\]/d' tests
replace_rs 's/#\[smol_potat::test\]/#\[test\]/' src
replace_rs '/#\[smol_potat::main\]/d' src
replace_rs '/#\[tokio::main\]/d' examples
replace_rs 's/#\[tokio::test\]/#\[test\]/' src
replace_rs 's/#\[cfg(feature = "sqlx-sqlite")\]/#\[cfg(feature = "rusqlite")\]/' tests
replace_rs '/[a-zA-Z]+<Self>: Send + Sync,/d' src
replace_rs '/[a-zA-Z]+<Self>: Send,/d' src
replace_rs 's/: Send + Sync {/ {/' src
replace_rs "s/type Stream<'a>: Stream/type Stream<'a>: Iterator/" src
replace_rs 's/: Send {/ {/' src
replace_rs 's/>: Send$/>/' src
replace_rs 's/: Sync {/ {/' src
replace_rs 's/Send + Sync + //' src
replace_rs 's/ + Sync//' src
replace_rs 's/ + Send//' src
replace_rs 's/Send + //' src
replace_rs 's/Arc<dyn std::error::Error>/Arc<dyn std::error::Error + Send + Sync>/' src
replace_rs '/T: Send,/d' src
replace_rs '/R::Model: Send,/d' src
replace_rs '/S::Item: Send,/d' src
replace_rs 's/Box::pin/Box::new/' src
replace_rs 's/impl Stream</impl Iterator</' src
replace_rs 's/S: Stream</S: Iterator</' src
replace_rs 's/use futures_util::lock::Mutex;/use std::sync::Mutex;/' src
replace_rs 's/use futures_util::lock::MutexGuard;/use std::sync::MutexGuard;/' src
replace_rs 's/, pin::Pin//' src
replace_rs 's/{pin::Pin, /{/' src
replace_rs 's/{task::Poll, /{/' src
replace_rs 's/{future::Future, /{/' src
replace_rs 's/, task::Poll//' src
replace_rs '/use std::{pin::Pin};/d' src
replace_rs '/use std::{task::Poll};/d' src
replace_rs '/use std::{future::Future};/d' src
replace_rs 's/, future::Future//' src
replace_rs '/use futures_util::Stream/d' src
replace_rs '/use futures_util::{Stream/d' src
replace_rs '/use futures_util::{TryStreamExt,/d' src
replace_rs '/\/\/\/ use futures_util/d' src
replace_rs '/use futures_util::future::BoxFuture;/d' src
replace_rs '/use async_stream::/d' src
replace_rs '/use futures_util::StreamExt/d' tests
replace_rs 's/self.conn.try_lock()/self.conn.try_lock().ok()/' src

sed_in_place 's/self.conn.try_lock().ok()/self.conn.try_lock()/' ./src/driver/rusqlite.rs
cargo +nightly fmt
