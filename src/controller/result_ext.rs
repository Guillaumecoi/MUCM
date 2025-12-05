//! Result extension for converting to DisplayResult
//!
//! Provides convenient methods to convert Result types to DisplayResult,
//! eliminating boilerplate error handling in controllers.

use super::DisplayResult;
use anyhow::Result;

/// Extension trait for Result to easily convert to DisplayResult
pub trait ResultExt<T> {
    /// Convert Result to DisplayResult with a success message
    ///
    /// # Arguments
    /// * `success_msg` - Message to display on success
    ///
    /// # Examples
    /// ```ignore
    /// self.app_service.create_actor(...)
    ///     .to_display("✅ Actor created successfully")
    /// ```
    fn to_display(self, success_msg: impl Into<String>) -> Result<DisplayResult>;

    /// Convert Result to DisplayResult with a formatted success message
    ///
    /// # Arguments
    /// * `f` - Closure that takes the success value and returns a message
    ///
    /// # Examples
    /// ```ignore
    /// self.app_service.create_actor(...)
    ///     .to_display_with(|id| format!("✅ Created actor: {}", id))
    /// ```
    fn to_display_with<F>(self, f: F) -> Result<DisplayResult>
    where
        F: FnOnce(&T) -> String;
}

impl<T> ResultExt<T> for Result<T> {
    fn to_display(self, success_msg: impl Into<String>) -> Result<DisplayResult> {
        match self {
            Ok(_) => Ok(DisplayResult::success(success_msg.into())),
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }

    fn to_display_with<F>(self, f: F) -> Result<DisplayResult>
    where
        F: FnOnce(&T) -> String,
    {
        match self {
            Ok(ref value) => Ok(DisplayResult::success(f(value))),
            Err(e) => Ok(DisplayResult::error(e.to_string())),
        }
    }
}

/// Helper for creating common success messages
pub struct DisplayMessage;

impl DisplayMessage {
    pub fn created(entity: &str, id: &str, name: &str) -> String {
        format!("✅ Created {}: {} - {}", entity, id, name)
    }

    pub fn created_simple(entity: &str, id: &str) -> String {
        format!("✅ Created {}: {}", entity, id)
    }

    pub fn updated(entity: &str, id: &str) -> String {
        format!("✅ Updated {}: {}", entity, id)
    }

    pub fn deleted(entity: &str, id: &str) -> String {
        format!("🗑️  Deleted {}: {}", entity, id)
    }

    pub fn added(what: &str, to: &str) -> String {
        format!("✅ Added {} to {}", what, to)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_display_success() {
        let result: Result<String> = Ok("test".to_string());
        let display = result.to_display("Success!").unwrap();
        assert!(display.is_success());
        assert_eq!(display.message, "Success!");
    }

    #[test]
    fn test_to_display_error() {
        let result: Result<String> = Err(anyhow::anyhow!("Test error"));
        let display = result.to_display("Success!").unwrap();
        assert!(!display.is_success());
        assert_eq!(display.message, "Test error");
    }

    #[test]
    fn test_to_display_with() {
        let result: Result<String> = Ok("actor-123".to_string());
        let display = result
            .to_display_with(|id| format!("✅ Created: {}", id))
            .unwrap();
        assert!(display.is_success());
        assert_eq!(display.message, "✅ Created: actor-123");
    }

    #[test]
    fn test_display_message_helpers() {
        assert_eq!(
            DisplayMessage::created("actor", "id-1", "Test"),
            "✅ Created actor: id-1 - Test"
        );
        assert_eq!(
            DisplayMessage::created_simple("use case", "UC-001"),
            "✅ Created use case: UC-001"
        );
        assert_eq!(
            DisplayMessage::updated("scenario", "S01"),
            "✅ Updated scenario: S01"
        );
        assert_eq!(
            DisplayMessage::deleted("persona", "john-doe"),
            "🗑️  Deleted persona: john-doe"
        );
        assert_eq!(
            DisplayMessage::added("step", "scenario S01"),
            "✅ Added step to scenario S01"
        );
    }

    #[test]
    fn test_to_display_with_error() {
        let result: Result<u32> = Err(anyhow::anyhow!("Database connection failed"));
        let display = result
            .to_display_with(|num| format!("Processed {} items", num))
            .unwrap();
        assert!(!display.is_success());
        assert_eq!(display.message, "Database connection failed");
    }

    #[test]
    fn test_to_display_chain() {
        fn create_user(name: &str) -> Result<String> {
            if name.is_empty() {
                Err(anyhow::anyhow!("Name cannot be empty"))
            } else {
                Ok(format!("user-{}", name))
            }
        }

        // Success case
        let result = create_user("alice").to_display("User created");
        assert!(result.unwrap().is_success());

        // Error case
        let result = create_user("").to_display("User created");
        assert!(!result.unwrap().is_success());
    }

    #[test]
    fn test_display_message_with_special_characters() {
        assert_eq!(
            DisplayMessage::created("actor", "test-id-123", "Name with spaces"),
            "✅ Created actor: test-id-123 - Name with spaces"
        );
        assert_eq!(
            DisplayMessage::added("reference (includes)", "use case UC-001"),
            "✅ Added reference (includes) to use case UC-001"
        );
    }
}
