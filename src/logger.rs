use env_logger::{Builder, Env};
use log::LevelFilter;

/// Initialize the logger with the specified verbosity level
pub fn init(verbosity: u8) {
    let env = Env::default();

    let mut builder = Builder::from_env(env);

    // Set log level based on verbosity flag
    let level = match verbosity {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    builder
        .filter_level(level)
        .format_timestamp(None) // Don't include timestamps in logs
        .format_module_path(false) // Don't include module path
        .init();
}
