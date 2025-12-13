rm -rf sea-orm-sync/src
rm -rf sea-orm-sync/tests
cp -r src sea-orm-sync
cp -r tests sea-orm-sync/
rm -rf sea-orm-sync/src/bin
cd sea-orm-sync
find src -type f -name '*.rs' -exec sed -i '' "s/Pin<Box<dyn Future<Output = Result<Self::Stream<'a>, DbErr>> + 'a + Send>>/Result<Self::Stream<'a>, DbErr>/" {} +
find src -type f -name '*.rs' -exec sed -i '' "s/Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'b>>/Result<T, E>/" {} +
find src -type f -name '*.rs' -exec sed -i '' "s/Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>/Result<T, E>/" {} +
find src   -type f -name '*.rs' -exec sed -i '' 's/Box::pin(async move {/({/' {} +
find tests -type f -name '*.rs' -exec sed -i '' 's/Box::pin(async move {/({/' {} +
find src   -type f -name '*.rs' -exec sed -i '' 's/async //' {} +
find tests -type f -name '*.rs' -exec sed -i '' 's/async //' {} +
find src   -type f -name '*.rs' -exec sed -i '' 's/\.await//' {} +
find tests -type f -name '*.rs' -exec sed -i '' 's/\.await//' {} +
find src   -type f -name '*.rs' -exec sed -i '' '/#\[async_trait::async_trait\]/d' {} +
find tests -type f -name '*.rs' -exec sed -i '' '/#\[async_trait::async_trait\]/d' {} +
find src -type f -name '*.rs' -exec sed -i '' 's/#\[smol_potat::test\]/#\[test\]/' {} +
find src -type f -name '*.rs' -exec sed -i '' '/#\[smol_potat::main\]/d' {} +
find src -type f -name '*.rs' -exec sed -i '' 's/#\[tokio::test\]/#\[test\]/' {} +
find tests -type f -name '*.rs' -exec sed -i '' 's/#\[cfg(feature = "sqlx-sqlite")\]/#\[cfg(feature = "rusqlite")\]/' {} +
find src -type f -name '*.rs' -exec sed -i '' "/[a-zA-Z]+<Self>: Send + Sync,/d" {} +
find src -type f -name '*.rs' -exec sed -i '' "/[a-zA-Z]+<Self>: Send,/d" {} +
find src -type f -name '*.rs' -exec sed -i '' "s/: Send + Sync {/ {/" {} +
find src -type f -name '*.rs' -exec sed -i '' "s/type Stream<'a>: Stream/type Stream<'a>: Iterator/" {} +
find src -type f -name '*.rs' -exec sed -i '' "s/: Send {/ {/" {} +
find src -type f -name '*.rs' -exec sed -i '' "s/: Sync {/ {/" {} +
find src -type f -name '*.rs' -exec sed -i '' "s/Send + Sync + //" {} +
find src -type f -name '*.rs' -exec sed -i '' "s/ + Sync//" {} +
find src -type f -name '*.rs' -exec sed -i '' "s/ + Send//" {} +
find src -type f -name '*.rs' -exec sed -i '' "s/Send + //" {} +
find src -type f -name '*.rs' -exec sed -i '' '/T: Send,/d' {} +
find src -type f -name '*.rs' -exec sed -i '' '/R::Model: Send,/d' {} +
find src -type f -name '*.rs' -exec sed -i '' '/S::Item: Send,/d' {} +
find src -type f -name '*.rs' -exec sed -i '' "s/Box::pin/Box::new/" {} +
find src -type f -name '*.rs' -exec sed -i '' 's/impl Stream</impl Iterator</' {} +
find src -type f -name '*.rs' -exec sed -i '' 's/S: Stream</S: Iterator</' {} +
find src -type f -name '*.rs' -exec sed -i '' 's/use futures_util::lock::Mutex;/use std::sync::Mutex;/' {} +
find src -type f -name '*.rs' -exec sed -i '' 's/use futures_util::lock::MutexGuard;/use std::sync::MutexGuard;/' {} +
find src -type f -name '*.rs' -exec sed -i '' 's/, pin::Pin//' {} +
find src -type f -name '*.rs' -exec sed -i '' 's/, future::Future//' {} +
find src -type f -name '*.rs' -exec sed -i '' '/use futures_util::Stream/d' {} +
find src -type f -name '*.rs' -exec sed -i '' '/use futures_util::{Stream/d' {} +
find src -type f -name '*.rs' -exec sed -i '' '/use futures_util::{TryStreamExt,/d' {} +
find src -type f -name '*.rs' -exec sed -i '' '/\/\/\/ use futures_util/d' {} +
find src -type f -name '*.rs' -exec sed -i '' '/use futures_util::future::BoxFuture;/d' {} +
find src -type f -name '*.rs' -exec sed -i '' '/use async_stream::/d' {} +
find tests -type f -name '*.rs' -exec sed -i '' '/use futures_util::StreamExt/d' {} +
find src -type f -name '*.rs' -exec sed -i '' 's/self.conn.try_lock()/self.conn.try_lock().ok()/' {} +
sed -i '' 's/self.conn.try_lock().ok()/self.conn.try_lock()/' ./src/driver/rusqlite.rs
cargo fmt