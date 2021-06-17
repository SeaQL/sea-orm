#[macro_export]
#[cfg(feature = "debug-print")]
macro_rules! debug_print {
    ($( $args:expr ),*) => { println!( $( $args ),* ); }
}

#[macro_export]
// Non-debug version
#[cfg(not(feature = "debug-print"))]
macro_rules! debug_print {
    ($( $args:expr ),*) => {
        true;
    };
}
