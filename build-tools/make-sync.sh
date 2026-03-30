#!/usr/bin/env bash
set -euo pipefail

# macOS (BSD) sed requires an empty-string argument after -i; GNU sed does not
if sed --version 2>/dev/null | grep -q GNU; then
    SI=(sed -i)
else
    SI=(sed -i '')
fi

rm -rf sea-orm-sync/src
rm -rf sea-orm-sync/tests
cp -r src sea-orm-sync
cp -r tests sea-orm-sync
cp -r examples/quickstart/src/main.rs sea-orm-sync/examples/quickstart/src/main.rs
rm -rf sea-orm-sync/src/bin
cd sea-orm-sync
find src -type f -name '*.rs' -exec "${SI[@]}" "s/Pin<Box<dyn Future<Output = Result<Self::Stream<'a>, DbErr>> + 'a + Send>>/Result<Self::Stream<'a>, DbErr>/" {} +
find src -type f -name '*.rs' -exec "${SI[@]}" "s/Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'b>>/Result<T, E>/" {} +
find src -type f -name '*.rs' -exec "${SI[@]}" "s/Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>/Result<T, E>/" {} +
find src   -type f -name '*.rs' -exec "${SI[@]}" 's/Box::pin(async move {/({/' {} +
find tests -type f -name '*.rs' -exec "${SI[@]}" 's/Box::pin(async move {/({/' {} +
find src   -type f -name '*.rs' -exec "${SI[@]}" 's/async //' {} +
find tests -type f -name '*.rs' -exec "${SI[@]}" 's/async //' {} +
find examples -type f -name '*.rs' -exec "${SI[@]}" 's/async //' {} +
find src   -type f -name '*.rs' -exec "${SI[@]}" 's/\.await//' {} +
find tests -type f -name '*.rs' -exec "${SI[@]}" 's/\.await//' {} +
find examples -type f -name '*.rs' -exec "${SI[@]}" 's/\.await//' {} +
find src   -type f -name '*.rs' -exec "${SI[@]}" '/#\[async_trait::async_trait\]/d' {} +
find tests -type f -name '*.rs' -exec "${SI[@]}" '/#\[async_trait::async_trait\]/d' {} +
find src -type f -name '*.rs' -exec "${SI[@]}" 's/#\[smol_potat::test\]/#\[test\]/' {} +
find src -type f -name '*.rs' -exec "${SI[@]}" '/#\[smol_potat::main\]/d' {} +
find examples -type f -name '*.rs' -exec "${SI[@]}" '/#\[tokio::main\]/d' {} +
find src -type f -name '*.rs' -exec "${SI[@]}" 's/#\[tokio::test\]/#\[test\]/' {} +
find tests -type f -name '*.rs' -exec "${SI[@]}" 's/#\[cfg(feature = "sqlx-sqlite")\]/#\[cfg(feature = "rusqlite")\]/' {} +
find src -type f -name '*.rs' -exec "${SI[@]}" "/[a-zA-Z]+<Self>: Send + Sync,/d" {} +
find src -type f -name '*.rs' -exec "${SI[@]}" "/[a-zA-Z]+<Self>: Send,/d" {} +
find src -type f -name '*.rs' -exec "${SI[@]}" "s/: Send + Sync {/ {/" {} +
find src -type f -name '*.rs' -exec "${SI[@]}" "s/type Stream<'a>: Stream/type Stream<'a>: Iterator/" {} +
find src -type f -name '*.rs' -exec "${SI[@]}" "s/: Send {/ {/" {} +
find src -type f -name '*.rs' -exec "${SI[@]}" "s/>: Send$/>/" {} +
find src -type f -name '*.rs' -exec "${SI[@]}" "s/: Sync {/ {/" {} +
find src -type f -name '*.rs' -exec "${SI[@]}" "s/Send + Sync + //" {} +
find src -type f -name '*.rs' -exec "${SI[@]}" "s/ + Sync//" {} +
find src -type f -name '*.rs' -exec "${SI[@]}" "s/ + Send//" {} +
find src -type f -name '*.rs' -exec "${SI[@]}" "s/Send + //" {} +
find src -type f -name '*.rs' -exec "${SI[@]}" 's/Arc<dyn std::error::Error>/Arc<dyn std::error::Error + Send + Sync>/' {} +
find src -type f -name '*.rs' -exec "${SI[@]}" '/T: Send,/d' {} +
find src -type f -name '*.rs' -exec "${SI[@]}" '/R::Model: Send,/d' {} +
find src -type f -name '*.rs' -exec "${SI[@]}" '/S::Item: Send,/d' {} +
find src -type f -name '*.rs' -exec "${SI[@]}" "s/Box::pin/Box::new/" {} +
find src -type f -name '*.rs' -exec "${SI[@]}" 's/impl Stream</impl Iterator</' {} +
find src -type f -name '*.rs' -exec "${SI[@]}" 's/S: Stream</S: Iterator</' {} +
find src -type f -name '*.rs' -exec "${SI[@]}" 's/use futures_util::lock::Mutex;/use std::sync::Mutex;/' {} +
find src -type f -name '*.rs' -exec "${SI[@]}" 's/use futures_util::lock::MutexGuard;/use std::sync::MutexGuard;/' {} +
find src -type f -name '*.rs' -exec "${SI[@]}" 's/, pin::Pin//' {} +
find src -type f -name '*.rs' -exec "${SI[@]}" 's/{pin::Pin, /{/' {} +
find src -type f -name '*.rs' -exec "${SI[@]}" 's/{task::Poll, /{/' {} +
find src -type f -name '*.rs' -exec "${SI[@]}" 's/{future::Future, /{/' {} +
find src -type f -name '*.rs' -exec "${SI[@]}" 's/, task::Poll//' {} +
find src -type f -name '*.rs' -exec "${SI[@]}" '/use std::{pin::Pin};/d' {} +
find src -type f -name '*.rs' -exec "${SI[@]}" '/use std::{task::Poll};/d' {} +
find src -type f -name '*.rs' -exec "${SI[@]}" '/use std::{future::Future};/d' {} +
find src -type f -name '*.rs' -exec "${SI[@]}" 's/, future::Future//' {} +
find src -type f -name '*.rs' -exec "${SI[@]}" '/use futures_util::Stream/d' {} +
find src -type f -name '*.rs' -exec "${SI[@]}" '/use futures_util::{Stream/d' {} +
find src -type f -name '*.rs' -exec "${SI[@]}" '/use futures_util::{TryStreamExt,/d' {} +
find src -type f -name '*.rs' -exec "${SI[@]}" '/\/\/\/ use futures_util/d' {} +
find src -type f -name '*.rs' -exec "${SI[@]}" '/use futures_util::future::BoxFuture;/d' {} +
find src -type f -name '*.rs' -exec "${SI[@]}" '/use async_stream::/d' {} +
find tests -type f -name '*.rs' -exec "${SI[@]}" '/use futures_util::StreamExt/d' {} +
find src -type f -name '*.rs' -exec "${SI[@]}" 's/self.conn.try_lock()/self.conn.try_lock().ok()/' {} +
"${SI[@]}" 's/self.conn.try_lock().ok()/self.conn.try_lock()/' ./src/driver/rusqlite.rs
cargo fmt
cargo +nightly fmt