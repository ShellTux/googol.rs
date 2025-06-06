//! A set of macros for convenient variable debugging and logging with different styles.
//!
//! These macros generate debug, error, and info logs for a given variable,
//! optionally allowing the caller to specify the formatting style.
//!
//! # Macros
//! - `debugv!` : Logs a variable at debug level.
//! - `errorv!` : Logs a variable at error level.
//! - `infov!`   : Logs a variable at info level.
//!
//! # Usage
//! ```
//! use googol::debugv;
//! use log::debug;
//!
//! pretty_env_logger::init();
//!
//! let a = vec![1, 2, 3];
//! debugv!(a); // Uses the default Debug format
//! debugv!(a, debug); // Uses Debug format with pretty-print
//! ```
//!
//! # Formats
//! - When the style argument is omitted, the macros use the `Debug` trait with `{:?}`.
//! - When `display` is specified, they use the `Display` trait with `{}`.
//! - When `debug` is specified, they use the `Debug` trait with `{:#?}` (pretty-print).
//!
//! # Requirements
//! - The `log` crate should be included and properly initialized with a logger
//!   (e.g., `env_logger`) for these macros to produce output.
//!
//! # Note
//! These macros use `stringify!` to print the variable's name, followed by its value.

/// Logs a variable at the debug level with optional styling.
///
/// # Arguments
/// - `var`: The variable to log.
/// - `style` (optional): The style of formatting (`debug` or `display`).
///
/// # Examples
/// ```
/// use googol::debugv;
/// use log::debug;
///
/// pretty_env_logger::init();
///
/// let a = 42;
/// debugv!(a); // Uses Debug formatting
/// debugv!(a, display); // Uses Display formatting
/// debugv!(a, debug); // Uses pretty Debug formatting
/// ```
#[macro_export]
macro_rules! debugv {
    // Case when style is provided: e.g., debugv!(a, debug);
    ($var:expr, debug) => {
        debug!("{} = {:#?}", stringify!($var), $var);
    };
    ($var:expr, display) => {
        debug!("{} = {}", stringify!($var), $var);
    };
    // Default case: if style is not specified, use Debug
    ($var:expr) => {
        debug!("{} = {:?}", stringify!($var), $var);
    };
}

/// Logs a variable at the error level with optional styling.
///
/// # Arguments
/// - `var`: The variable to log.
/// - `style` (optional): The style of formatting (`debug` or `display`).
///
/// # Examples
/// ```
/// use googol::errorv;
/// use log::error;
///
/// pretty_env_logger::init();
///
/// let err_code = 404;
/// errorv!(err_code); // Uses Debug formatting
/// errorv!(err_code, display); // Uses Display formatting
/// errorv!(err_code, debug); // Uses pretty Debug formatting
/// ```
#[macro_export]
macro_rules! errorv {
    // Case when style is provided: e.g., errorv!(a, debug);
    ($var:expr, debug) => {
        error!("{} = {:#?}", stringify!($var), $var);
    };
    ($var:expr, display) => {
        error!("{} = {}", stringify!($var), $var);
    };
    // Default case: if style is not specified, use Debug
    ($var:expr) => {
        error!("{} = {:?}", stringify!($var), $var);
    };
}

/// Logs a variable at the info level with optional styling.
///
/// # Arguments
/// - `var`: The variable to log.
/// - `style` (optional): The style of formatting (`debug` or `display`).
///
/// # Examples
/// ```
/// use googol::infov;
/// use log::info;
///
/// pretty_env_logger::init();
///
/// let info = "Application started";
/// infov!(info); // Uses Debug formatting
/// infov!(info, display); // Uses Display formatting
/// infov!(info, debug); // Uses pretty Debug formatting
/// ```
#[macro_export]
macro_rules! infov {
    // Case when style is provided: e.g., infov!(a, debug);
    ($var:expr, debug) => {
        info!("{} = {:#?}", stringify!($var), $var);
    };
    ($var:expr, display) => {
        info!("{} = {}", stringify!($var), $var);
    };
    // Default case: if style is not specified, use Debug
    ($var:expr) => {
        info!("{} = {:?}", stringify!($var), $var);
    };
}
