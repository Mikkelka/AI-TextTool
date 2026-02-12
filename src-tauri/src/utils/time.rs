//! Time utilities for working with timestamps.
//!
//! This module provides helper functions for getting current timestamps
//! with proper error handling.

use std::time::{SystemTime, UNIX_EPOCH};

/// Get current timestamp in milliseconds since UNIX epoch
///
/// # Returns
/// * `u128` - Current timestamp in milliseconds, or 0 if system time is before UNIX_EPOCH
///
/// # Example
/// ```ignore
/// let timestamp = get_current_timestamp_millis();
/// println!("Current time: {}", timestamp);
/// ```
pub fn get_current_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_else(|e| {
            eprintln!("Failed to get current time: {}", e);
            0 // Fallback to 0 instead of panic
        })
}

/// Get current timestamp in seconds since UNIX epoch
///
/// # Returns
/// * `u64` - Current timestamp in seconds, or 0 if system time is before UNIX_EPOCH
#[allow(dead_code)]
pub fn get_current_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or_else(|e| {
            eprintln!("Failed to get current time: {}", e);
            0
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_current_timestamp_millis() {
        let timestamp = get_current_timestamp_millis();
        // Should be a reasonable timestamp (after year 2000)
        assert!(timestamp > 946_684_800_000); // Jan 1, 2000 in milliseconds
    }

    #[test]
    fn test_get_current_timestamp_secs() {
        let timestamp = get_current_timestamp_secs();
        // Should be a reasonable timestamp (after year 2000)
        assert!(timestamp > 946_684_800); // Jan 1, 2000 in seconds
    }
}
