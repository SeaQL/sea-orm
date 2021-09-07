#[macro_export]
#[cfg(feature = "debug-print")]
macro_rules! debug_print {
    ($( $args:expr ),*) => { log::debug!( $( $args ),* ); }
}

#[macro_export]
// Non-debug version
#[cfg(not(feature = "debug-print"))]
macro_rules! debug_print {
    ($( $args:expr ),*) => {
        true;
    };
}
