//! Date formatting utilities for markdown generation.

use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::HashMap;

/// Format dates from metadata and insert them into the data map.
///
/// This function extracts RFC3339 date strings from metadata, formats them
/// according to the provided date format, and inserts the formatted dates
/// as top-level fields in the data map.
///
/// # Arguments
/// * `data` - The data HashMap to insert formatted dates into
/// * `date_format` - The chrono format string to use (e.g., "%d/%m/%Y")
///
/// # Inserted Fields
/// - `created_date`: Formatted creation date
/// - `created`: Formatted creation date (alias)
/// - `last_updated`: Formatted last updated date
pub fn format_dates_from_metadata(data: &mut HashMap<String, Value>, date_format: &str) {
    if let Some(Value::Object(metadata)) = data.get("metadata").cloned() {
        // Format created_at
        if let Some(Value::String(created_at)) = metadata.get("created_at") {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(created_at) {
                let formatted = dt.format(date_format).to_string();
                data.insert("created_date".to_string(), Value::String(formatted.clone()));
                data.insert("created".to_string(), Value::String(formatted));
            }
        }

        // Format updated_at
        if let Some(Value::String(updated_at)) = metadata.get("updated_at") {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(updated_at) {
                let formatted = dt.format(date_format).to_string();
                data.insert("last_updated".to_string(), Value::String(formatted));
            }
        }
    }
}

/// Format DateTime<Utc> values directly into formatted strings.
///
/// This is useful when you have direct access to DateTime objects rather than
/// RFC3339 strings in JSON.
///
/// # Arguments
/// * `created_at` - The creation DateTime
/// * `updated_at` - The last updated DateTime
/// * `date_format` - The chrono format string to use (e.g., "%d/%m/%Y")
///
/// # Returns
/// A tuple of (created, last_updated) formatted date strings
pub fn format_datetime_pair(
    created_at: &DateTime<Utc>,
    updated_at: &DateTime<Utc>,
    date_format: &str,
) -> (String, String) {
    let created = created_at.format(date_format).to_string();
    let last_updated = updated_at.format(date_format).to_string();
    (created, last_updated)
}
