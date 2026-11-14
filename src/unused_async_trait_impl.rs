//! Regression case for `clippy::unused_async_trait_impl` false positive in https://github.com/rust-lang/rust-clippy/issues/17772.

// `unknown_lints` is allowed because the lint only exists on clippy newer than the MSRV.
#![allow(dead_code, unknown_lints)]

struct S;

impl S {
    #[allow(clippy::unused_async_trait_impl)]
    async fn get(&self) -> u32 {
        1
    }
}
