//! Regression case for `clippy::unused_async_trait_impl` false positive in https://github.com/rust-lang/rust-clippy/issues/17772.

#![allow(dead_code)]

struct S;

impl S {
    #[expect(clippy::unused_async_trait_impl)]
    async fn get(&self) -> u32 {
        1
    }
}
