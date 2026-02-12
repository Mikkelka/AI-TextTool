//! Validation utilities for input sanitation and checking.
//!
//! This module provides validation functions for user input to ensure
//! data integrity and security at API boundaries.

/// Maximum allowed length for text input (100KB)
pub const MAX_TEXT_LENGTH: usize = 100_000;

/// Minimum required length for text input
pub const MIN_TEXT_LENGTH: usize = 1;

/// Maximum allowed length for chat messages (10KB)
pub const MAX_MESSAGE_LENGTH: usize = 10_000;

/// Maximum allowed length for operation names
pub const MAX_OPERATION_NAME_LENGTH: usize = 50;

/// Validate text input for AI processing
///
/// # Arguments
/// * `text` - The text to validate
///
/// # Returns
/// * `Ok(())` if validation passes
/// * `Err(String)` with error message if validation fails
pub fn validate_text_input(text: &str) -> Result<(), String> {
    let trimmed = text.trim();

    if trimmed.is_empty() {
        return Err("Text cannot be empty".to_string());
    }

    if trimmed.len() < MIN_TEXT_LENGTH {
        return Err(format!("Text must be at least {} character(s)", MIN_TEXT_LENGTH));
    }

    if trimmed.len() > MAX_TEXT_LENGTH {
        return Err(format!(
            "Text cannot exceed {} characters ({} KB)",
            MAX_TEXT_LENGTH,
            MAX_TEXT_LENGTH / 1024
        ));
    }

    Ok(())
}

/// Validate chat message input
///
/// # Arguments
/// * `message` - The message to validate
///
/// # Returns
/// * `Ok(())` if validation passes
/// * `Err(String)` with error message if validation fails
pub fn validate_message_input(message: &str) -> Result<(), String> {
    let trimmed = message.trim();

    if trimmed.is_empty() {
        return Err("Message cannot be empty".to_string());
    }

    if trimmed.len() > MAX_MESSAGE_LENGTH {
        return Err(format!(
            "Message cannot exceed {} characters ({} KB)",
            MAX_MESSAGE_LENGTH,
            MAX_MESSAGE_LENGTH / 1024
        ));
    }

    Ok(())
}

/// Validate operation name
///
/// # Arguments
/// * `operation` - The operation name to validate
///
/// # Returns
/// * `Ok(())` if validation passes
/// * `Err(String)` with error message if validation fails
pub fn validate_operation_name(operation: &str) -> Result<(), String> {
    let trimmed = operation.trim();

    if trimmed.is_empty() {
        return Err("Operation name cannot be empty".to_string());
    }

    if trimmed.len() > MAX_OPERATION_NAME_LENGTH {
        return Err(format!("Operation name cannot exceed {} characters", MAX_OPERATION_NAME_LENGTH));
    }

    // Check for valid characters (alphanumeric, spaces, hyphens, underscores)
    if !trimmed
        .chars()
        .all(|c| c.is_alphanumeric() || c.is_whitespace() || c == '-' || c == '_')
    {
        return Err("Operation name contains invalid characters".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_text_input_valid() {
        assert!(validate_text_input("Hello world").is_ok());
        assert!(validate_text_input("  Valid text  ").is_ok());
    }

    #[test]
    fn test_validate_text_input_empty() {
        assert!(validate_text_input("").is_err());
        assert!(validate_text_input("   ").is_err());
    }

    #[test]
    fn test_validate_text_input_too_long() {
        let long_text = "a".repeat(MAX_TEXT_LENGTH + 1);
        assert!(validate_text_input(&long_text).is_err());
    }

    #[test]
    fn test_validate_message_input_valid() {
        assert!(validate_message_input("Hello").is_ok());
    }

    #[test]
    fn test_validate_operation_name_valid() {
        assert!(validate_operation_name("Proofread").is_ok());
        assert!(validate_operation_name("Custom-Operation").is_ok());
        assert!(validate_operation_name("test_op_123").is_ok());
    }

    #[test]
    fn test_validate_operation_name_invalid_chars() {
        assert!(validate_operation_name("invalid@name").is_err());
        assert!(validate_operation_name("bad!op").is_err());
    }
}
