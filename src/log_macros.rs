// Disable warnings
#[allow(unused_macros)]

macro_rules! log {
    ($( $args:expr ),*) => { println!( $( $args ),* ); }
}

macro_rules! debug_log {
    ($( $args:expr ),*) => {
        if cfg!(debug_assertions) {
            println!( $( $args ),* );
        }
    }
}
