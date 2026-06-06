//! Compile-fail harness for `Id<E, T>`-wrapped primary keys.
//!
//! Each fixture under `tests/value_type_pk_compile_fail/` must fail to
//! compile. This harness asserts only the must-fail invariant; the captured
//! stderr is regenerated on every run via `TRYBUILD=overwrite` and the
//! `.stderr` files are gitignored, so the exact compiler wording is not
//! pinned (it varies across rustc releases).

#[test]
fn pk_safety() {
    // SAFETY: env vars touch global state visible to other threads, but
    // this test runs synchronously and the only consumer of `TRYBUILD`
    // is trybuild itself, invoked on the line below.
    unsafe {
        std::env::set_var("TRYBUILD", "overwrite");
    }
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/value_type_pk_compile_fail/*.rs");
}
