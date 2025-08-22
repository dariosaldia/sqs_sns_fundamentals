use tracing_subscriber::{EnvFilter, fmt};

pub fn init() {
    // RUST_LOG=info,debug,...  (defaults to info)
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(filter).without_time().init();
}
