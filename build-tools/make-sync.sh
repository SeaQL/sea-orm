#!/usr/bin/env bash
set -euo pipefail

# macOS (BSD) sed requires an empty-string argument after -i; GNU sed does not
if sed --version 2>/dev/null | grep -q GNU; then
    SI=(sed -i)
else
    SI=(sed -i '')
fi

replace_rs() {
    local expression="$1"
    shift
    if [ "$#" -eq 0 ]; then
        set -- src tests examples
    fi
    find "$@" -type f -name '*.rs' -exec "${SI[@]}" "$expression" {} +
}

rm -rf sea-orm-sync/src sea-orm-sync/tests
cp -r src tests sea-orm-sync
cp examples/quickstart/src/main.rs sea-orm-sync/examples/quickstart/src/main.rs
rm -rf sea-orm-sync/src/bin
cd sea-orm-sync

# Keep the async-only metric test module available to rustfmt without generating dead tests.
printf '%s\n' '// Async-only metric tests are omitted from the sync crate.' > src/database/stream/metric/tests.rs

# Sync row collection uses the synchronous query methods, without the async row source.
replace_rs '
/^    \/\/\/ Query rows for internal model collection\.$/d
/^    \/\/\/ Query rows from a statement builder for internal model collection\.$/d
/^    #\[cfg(not(feature = "sync"))\]$/ {
    N
    /\n    #\[doc(hidden)\]$/N
    /\n    \(async \)\{0,1\}fn query_rows_raw(/b async_rows
    /\n    fn query_rows</b async_rows
    b
}
/^    #\[cfg(all($/ {
    :span_cfg
    N
    /\n    ))\]$/!b span_cfg
    N
    /\n    fn query_all_span(/!b
    b async_rows
}
b
:async_rows
N
/\n    }$/!b async_rows
d
' src/database/connection.rs src/database/db_connection.rs

# Erase future-only adapters before converting their bodies.
replace_rs '
s/\.left_future()//g
s/\.right_future()//g
'

# A Future's success mapping becomes Result's success mapping.
replace_rs 's/TryFutureExt::map_ok(/Result::map(/g'

# Async syntax.
replace_rs '
s/^\([[:space:]]*\)async \(move \)\{0,1\}{ \([^{;}]*\) }$/\1\3/
s/Box::pin(async move {/({/
s/async move {/{/

s/async fn /fn /g
s/async |/|/g
s/async {/{/g
s/\.await//g

/#\[async_trait::async_trait/d
s/transaction_with_config_async/transaction_with_config/g
s/transaction_async/transaction/g

s/#\[macro_rules_attribute::apply(smol_macros::test!)\]/#[test]/
/#\[macro_rules_attribute::apply(smol_macros::main!)\]/d
s/#\[tokio::test\]/#[test]/
/#\[tokio::main\]/d
'

# Join return types split by rustfmt before removing the Future wrapper.
replace_rs '
/impl Future<$/ {
:future
N
/\n[[:space:]]*>$/!b future
s/\n[[:space:]]*/ /g
s/impl Future< Output = \(.*\), >$/\1/
}
s/impl Future<Output = \(.*\)> {$/\1 {/
s/impl Future<Output = \(.*\)>$/\1/
s/AsyncFnOnce/FnOnce/g
'
replace_rs "
s/Pin<Box<dyn Future<Output = Result<Self::Stream<'a>, DbErr>> + 'a + Send>>/Result<Self::Stream<'a>, DbErr>/
s/Pin<Box<dyn Future<Output = Result<T, E>> + Send + '[bc]>>/Result<T, E>/
"
# Convert the known single-line types, then reject unsupported forms.
replace_rs "
s/\([>:] \)BoxFuture<'[[:alnum:]_]*, \(Result<[^<>]*<[^<>]*>[^<>]*>\)>/\1\2/g
s/\([>:] \)BoxFuture<'[[:alnum:]_]*, \(Result<[^<>]*>\)>/\1\2/g
"
if grep -RnE '(^|[^[:alnum:]_:])BoxFuture([[:space:]]*<|[[:space:]]*$)' src tests examples; then
    echo 'Unsupported internal BoxFuture form' >&2
    exit 1
else
    status=$?
    if [ "$status" -ne 1 ]; then
        exit "$status"
    fi
fi

# The synchronous library API does not require async thread bounds.
replace_rs "
s/: Send + Sync {/ {/
s/: Send {/ {/
s/>: Send$/>/
s/: Sync {/ {/
s/Send + Sync + //
s/ + Sync//
s/ + Send//
s/Send + //
s/Arc<dyn std::error::Error>/Arc<dyn std::error::Error + Send + Sync>/
/T: Send,/d
/R::Model: Send,/d
/S::Item: Send,/d
" src

# Stream syntax and async imports.
replace_rs "
s/futures_util::stream::once({/std::iter::once({/
s/type Stream<'a>: Stream/type Stream<'a>: Iterator/
s/Box::pin/Box::new/
s/impl Stream</impl Iterator</
s/S: Stream</S: Iterator</
s/use futures_util::lock::Mutex;/use std::sync::Mutex;/
s/use futures_util::lock::MutexGuard;/use std::sync::MutexGuard;/
s/, pin::Pin//
s/{pin::Pin, /{/
s/{task::Poll, /{/
s/{future::Future, /{/
s/, task::Poll//
/use std::{task::Poll};/d
/use std::{future::Future};/d
/use std::future::Future;/d
s/, future::Future//
"
# Remove complete imports made up only of async tools.
replace_rs '
/^[[:space:]]*use futures_util::FutureExt;$/d
/^[[:space:]]*use futures_util::TryFutureExt;$/d
/^[[:space:]]*use futures_util::future::ready;$/d
/^[[:space:]]*use futures_util::future::{OptionFuture, ready};$/d
/^[[:space:]]*use futures_util::{FutureExt, TryFutureExt};$/d
/^[[:space:]]*use futures_util::{FutureExt, StreamExt, TryFutureExt, TryStreamExt};$/d
/^[[:space:]]*use futures_util::stream;$/d
/^[[:space:]]*use futures_util::stream::{self, BoxStream};$/d
/^[[:space:]]*use std::task::{Context, Poll};$/d
'

# Remove gated async imports together with their attributes.
replace_rs '/^#\[cfg(.*)\]$/{N;/use futures_util::Stream\(Ext\)\{0,1\};/d;/use futures_util::TryFutureExt;/d;}'
replace_rs '
/use futures_util::Stream/d
/use futures_util::{Stream/d
/use futures_util::{TryStreamExt,/d
/use async_stream::/d
'
replace_rs '
/^#\[cfg(not(feature = "sync"))\]$/{N;/use .*BoxFuture;/d;}
/^pub(crate) use futures_util::future::BoxFuture;$/d
/^use super::BoxFuture;$/d
s/{BoxFuture, /{/
s/, database::BoxFuture//
'

# Documentation of the generated synchronous API.
replace_rs '
s/Convert self into an async stream/Convert self into a stream/
/\/\/\/ use futures_util::TryStreamExt;/d
s/Execute the async function inside a transaction\./Execute the function inside a transaction./
s/next async operation/next operation/
s/async support from day 1/support from day 1/
'

replace_rs '
s|^/// Execute a future and wrap it in a tracing span when `tracing-spans` is enabled\.$|/// Evaluate an expression inside a tracing span when `tracing-spans` is enabled.|
s|^/// When the feature is disabled, this macro simply awaits the future with zero overhead\.$|/// When the feature is disabled, evaluate the expression directly.|
s|^/// - `\$fut`: the future to execute$|/// - `$fut` is the expression to evaluate.|
' src/database/tracing_spans.rs

cargo +nightly fmt
