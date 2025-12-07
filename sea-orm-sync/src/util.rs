/// Uses the `log` crate to perform logging.
/// This must be enabled using the feature flag `debug-print`.
/// ### Usage
/// ```
/// use sea_orm::debug_print;
///
/// #[derive(Debug)]
/// enum FooError {
///     Bar,
///     Baz,
/// }
///
/// debug_print!("{:?}", FooError::Bar);
/// ```
#[macro_export]
#[cfg(feature = "debug-print")]
macro_rules! debug_print {
    ($( $args:expr ),*) => { tracing::debug!( $( $args ),* ); }
}

#[macro_export]
/// Non-debug version
#[cfg(not(feature = "debug-print"))]
macro_rules! debug_print {
    ($( $args:expr ),*) => {
        true;
    };
}

#[cfg(all(test, feature = "sync"))]
pub trait StreamShim<T> {
    fn try_next(&mut self) -> Result<Option<T>, crate::DbErr>;
}

#[cfg(all(test, feature = "sync"))]
impl<I, T> StreamShim<T> for I
where
    I: Iterator<Item = Result<T, crate::DbErr>>,
{
    fn try_next(&mut self) -> Result<Option<T>, crate::DbErr> {
        self.next().transpose()
    }
}
