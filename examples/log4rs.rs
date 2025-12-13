use log::{debug, error, info, trace, warn};

fn main() {
    log4rs::init_file("examples/config/log4rs/log4rs.yaml", Default::default()).unwrap();

    debug!("debug");
    error!("error");
    info!("info");
    trace!("trace");
    warn!("warn");
}
