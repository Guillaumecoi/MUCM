//! # Field Input Helpers
//!
//! Reusable field input functions for interactive editing of different field types.
//! These helpers provide type-aware input methods that can be used across
//! use cases, personas, scenarios, and other entities.

use anyhow::Result;
use inquire::{Confirm, Select, Text};
use serde_json::Value as JsonValue;

use super::ui::UI;

/// Helper functions for interactive field editing
pub struct FieldHelpers;

impl FieldHelpers {
    /// Edit an array field interactively with add/edit/remove/clear options
    ///
    /// # Arguments
    /// * `label` - Display name for the array field
    /// * `current_items` - Existing items in the array
    ///
    /// # Returns
    /// * `Ok(Some(items))` - Modified array items
    /// * `Ok(None)` - No changes made (user kept original)
    pub fn edit_array(label: &str, current_items: Vec<String>) -> Result<Option<Vec<String>>> {
        let original_items = current_items.clone();
        let mut items = current_items;

        loop {
            UI::show_info(&format!("\n📋 {} (current: [{}])", label, items.join(", ")))?;

            let options = if items.is_empty() {
                vec!["Add item", "Done (keep empty)"]
            } else {
                vec!["Add item", "Edit item", "Remove item", "Clear all", "Done"]
            };

            let choice = Select::new("Action:", options).prompt()?;

            match choice {
                "Add item" => {
                    let item = Text::new("  New item:").prompt()?;
                    if !item.trim().is_empty() {
                        items.push(item.trim().to_string());
                    }
                }
                "Edit item" => {
                    if items.is_empty() {
                        continue;
                    }

                    let item_to_edit =
                        Select::new("Select item to edit:", items.clone()).prompt()?;
                    let idx = items.iter().position(|i| i == &item_to_edit).unwrap();

                    let new_value = Text::new(&format!("  Edit '{}' to:", item_to_edit))
                        .with_default(&item_to_edit)
                        .prompt()?;

                    if !new_value.trim().is_empty() {
                        items[idx] = new_value.trim().to_string();
                    }
                }
                "Remove item" => {
                    if items.is_empty() {
                        continue;
                    }

                    let item_to_remove =
                        Select::new("Select item to remove:", items.clone()).prompt()?;
                    items.retain(|i| i != &item_to_remove);
                }
                "Clear all" => {
                    let confirm = Confirm::new("Clear all items?")
                        .with_default(false)
                        .prompt()?;
                    if confirm {
                        items.clear();
                    }
                }
                "Done" | "Done (keep empty)" => break,
                _ => {}
            }
        }

        // Return None if no changes were made
        if items == original_items {
            Ok(None)
        } else {
            Ok(Some(items))
        }
    }

    /// Prompt for a number with validation
    ///
    /// # Arguments
    /// * `label` - Display text for the prompt
    /// * `current` - Current value as string
    /// * `help` - Help message to display
    ///
    /// # Returns
    /// * `Ok(Some(value))` - New validated number as string
    /// * `Ok(None)` - No change (kept original or empty)
    pub fn edit_number(label: &str, current: &str, help: &str) -> Result<Option<String>> {
        loop {
            let result = Text::new(label)
                .with_default(current)
                .with_help_message(help)
                .prompt()?;

            if result.trim().is_empty() || result == current {
                return Ok(None);
            }

            if result.parse::<f64>().is_ok() {
                return Ok(Some(result));
            } else {
                UI::show_error("Please enter a valid number")?;
            }
        }
    }

    /// Prompt for a boolean value
    ///
    /// # Arguments
    /// * `label` - Display text for the prompt
    /// * `current` - Current boolean value
    /// * `help` - Help message to display
    ///
    /// # Returns
    /// * `Ok(Some(value))` - New boolean as string if changed
    /// * `Ok(None)` - No change (kept original)
    pub fn edit_boolean(label: &str, current: bool, help: &str) -> Result<Option<String>> {
        let result = Confirm::new(label)
            .with_default(current)
            .with_help_message(help)
            .prompt()?;

        if result != current {
            Ok(Some(result.to_string()))
        } else {
            Ok(None)
        }
    }

    /// Prompt for a string value
    ///
    /// # Arguments
    /// * `label` - Display text for the prompt
    /// * `current` - Current string value
    /// * `help` - Help message to display
    ///
    /// # Returns
    /// * `Ok(Some(value))` - New string if changed
    /// * `Ok(None)` - No change (kept original)
    pub fn edit_string(label: &str, current: &str, help: &str) -> Result<Option<String>> {
        let new_value = Text::new(label)
            .with_default(current)
            .with_help_message(help)
            .prompt()?;

        if new_value != current {
            Ok(Some(new_value))
        } else {
            Ok(None)
        }
    }

    /// Parse JSON value as array of strings
    ///
    /// # Arguments
    /// * `json_val` - JSON value that might be an array
    ///
    /// # Returns
    /// Vector of strings extracted from the JSON value
    pub fn parse_json_array(json_val: &JsonValue) -> Vec<String> {
        if let Some(arr) = json_val.as_array() {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        } else if let Some(s) = json_val.as_str() {
            // Handle string stored as array (legacy format with newline-separated values)
            s.split('\n')
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Convert vector of strings to storage format (newline-separated)
    ///
    /// # Arguments
    /// * `items` - Vector of string items
    ///
    /// # Returns
    /// String with items joined by newlines
    pub fn array_to_storage(items: &[String]) -> String {
        items.join("\n")
    }

    /// Edit a field based on its type definition
    ///
    /// This is a high-level helper that automatically chooses the appropriate
    /// input method based on the field type.
    ///
    /// # Arguments
    /// * `field_type` - Type of the field ("string", "number", "boolean", "array", "text")
    /// * `label` - Display name for the field
    /// * `current_value` - Current JSON value of the field (if any)
    /// * `help` - Help message to display
    ///
    /// # Returns
    /// * `Ok(Some(value))` - New value as string if changed
    /// * `Ok(None)` - No change made
    pub fn edit_by_type(
        field_type: &str,
        label: &str,
        current_value: Option<&JsonValue>,
        help: &str,
    ) -> Result<Option<String>> {
        match field_type {
            "array" => {
                let current_items = current_value
                    .map(Self::parse_json_array)
                    .unwrap_or_default();

                if let Some(items) = Self::edit_array(label, current_items)? {
                    Ok(Some(Self::array_to_storage(&items)))
                } else {
                    Ok(None)
                }
            }
            "number" => {
                let current = current_value.map(|v| format!("{}", v)).unwrap_or_default();

                Self::edit_number(&format!("{}: ", label), &current, help)
            }
            "boolean" => {
                let current_bool = current_value.and_then(|v| v.as_bool()).unwrap_or(false);

                Self::edit_boolean(&format!("{}: ", label), current_bool, help)
            }
            _ => {
                // "text" and default to string
                let current = current_value.and_then(|v| v.as_str()).unwrap_or("");

                Self::edit_string(&format!("{}: ", label), current, help)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_json_array() {
        let json_arr = json!(["item1", "item2", "item3"]);
        let result = FieldHelpers::parse_json_array(&json_arr);
        assert_eq!(result, vec!["item1", "item2", "item3"]);
    }

    #[test]
    fn test_parse_json_array_from_string() {
        let json_str = json!("item1\nitem2\nitem3");
        let result = FieldHelpers::parse_json_array(&json_str);
        assert_eq!(result, vec!["item1", "item2", "item3"]);
    }

    #[test]
    fn test_parse_json_array_empty() {
        let json_empty = json!("");
        let result = FieldHelpers::parse_json_array(&json_empty);
        assert!(result.is_empty());
    }

    #[test]
    fn test_array_to_storage() {
        let items = vec![
            "item1".to_string(),
            "item2".to_string(),
            "item3".to_string(),
        ];
        let result = FieldHelpers::array_to_storage(&items);
        assert_eq!(result, "item1\nitem2\nitem3");
    }

    #[test]
    fn test_array_to_storage_empty() {
        let items: Vec<String> = vec![];
        let result = FieldHelpers::array_to_storage(&items);
        assert_eq!(result, "");
    }
}
