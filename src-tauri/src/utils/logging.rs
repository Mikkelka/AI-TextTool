//! Logging setup and contract for backend diagnostics.
//!
//! Contract:
//! - `debug!`: verbose diagnostic details
//! - `info!`: normal lifecycle events
//! - `warn!`: recoverable issues/fallback paths
//! - `error!`: actionable failures

use env_logger::{Builder, Env};
use std::sync::Once;

static LOGGING_INIT: Once = Once::new();

/// Initialize global logger once for the entire app process.
pub fn init_logging() {
    LOGGING_INIT.call_once(|| {
        let default_level = if cfg!(debug_assertions) {
            "debug"
        } else {
            "info"
        };

        let _ = Builder::from_env(Env::default().default_filter_or(default_level))
            .format_timestamp_millis()
            .try_init();
    });
}
