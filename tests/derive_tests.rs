use sea_orm::{FromQueryResult, TryGetable};

#[derive(FromQueryResult)]
struct SimpleTest {
    _foo: i32,
    _bar: String,
}

#[derive(FromQueryResult)]
struct GenericTest<T> {
    _foo: i32,
    _bar: T,
}

#[derive(FromQueryResult)]
struct DoubleGenericTest<T, F> {
    _foo: T,
    _bar: F,
}

#[derive(FromQueryResult)]
struct BoundsGenericTest<T: Copy + Clone + 'static> {
    _foo: T,
}

#[derive(FromQueryResult)]
struct WhereGenericTest<T>
where
    T: Copy + Clone + 'static,
{
    _foo: T,
}

#[derive(FromQueryResult)]
struct AlreadySpecifiedBoundsGenericTest<T: TryGetable> {
    _foo: T,
}

#[derive(FromQueryResult)]
struct MixedGenericTest<T: Clone, F>
where
    F: Copy + Clone + 'static,
{
    _foo: T,
    _bar: F,
}
