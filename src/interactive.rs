#[macro_export]
macro_rules! wait_for_enter {
    ($($arg:tt)*) => {{
        use std::io::{self, Write};
        // Print the formatted message
        print!($($arg)*);
        // Ensure the message appears immediately
        io::stdout().flush().unwrap();
        // Wait for Enter key
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
    }};
}
