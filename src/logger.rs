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

#[macro_export]
macro_rules! errorv {
    // Case when style is provided: e.g., debugv!(a, debug);
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

#[macro_export]
macro_rules! infov {
    // Case when style is provided: e.g., debugv!(a, debug);
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
