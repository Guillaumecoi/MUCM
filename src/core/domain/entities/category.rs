//! Category entity for organizing use cases
//!
//! Categories have both a full name and a unique uppercase abbreviation (minimum 3 characters).
//! Both names and abbreviations are case-insensitive for uniqueness checks.

use serde::{Deserialize, Serialize};

/// Represents a use case category with full name and abbreviation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Category {
    /// Full category name (e.g., "Authentication")
    pub full_name: String,
    /// Uppercase abbreviation for use in IDs (e.g., "AUT"), minimum 3 characters
    pub abbreviation: String,
}

impl Category {
    /// Create a new category with validation
    ///
    /// # Arguments
    /// * `full_name` - The full category name
    /// * `abbreviation` - The abbreviation (will be converted to uppercase)
    ///
    /// # Returns
    /// Result with the Category or an error message
    pub fn new(full_name: String, abbreviation: String) -> anyhow::Result<Self> {
        let trimmed_name = full_name.trim().to_string();
        let upper_abbrev = abbreviation.trim().to_uppercase();

        if trimmed_name.is_empty() {
            anyhow::bail!("Category name cannot be empty");
        }

        if !Self::is_valid_abbreviation(&upper_abbrev) {
            anyhow::bail!(
                "Abbreviation must be at least 3 uppercase letters, got: {}",
                upper_abbrev
            );
        }

        Ok(Category {
            full_name: trimmed_name,
            abbreviation: upper_abbrev,
        })
    }

    /// Check if an abbreviation is valid (at least 3 chars, all letters/numbers)
    pub fn is_valid_abbreviation(abbrev: &str) -> bool {
        abbrev.len() >= 3 && abbrev.chars().all(|c| c.is_alphanumeric())
    }

    /// Normalize a string for case-insensitive comparison
    pub fn normalize_for_comparison(s: &str) -> String {
        s.trim().to_lowercase()
    }

    /// Check if this category's name matches another (case-insensitive)
    pub fn name_matches(&self, other: &str) -> bool {
        Self::normalize_for_comparison(&self.full_name) == Self::normalize_for_comparison(other)
    }

    /// Check if this category's abbreviation matches another (case-insensitive)
    pub fn abbreviation_matches(&self, other: &str) -> bool {
        Self::normalize_for_comparison(&self.abbreviation) == Self::normalize_for_comparison(other)
    }

    /// Suggest an abbreviation from a category name (first 3 uppercase chars)
    pub fn suggest_abbreviation(name: &str) -> String {
        name.chars()
            .filter(|c| c.is_alphabetic())
            .take(3)
            .collect::<String>()
            .to_uppercase()
    }

    /// Find the first position where two strings differ (for collision resolution)
    pub fn find_first_difference(s1: &str, s2: &str) -> Option<usize> {
        let s1_lower = s1.to_lowercase();
        let s2_lower = s2.to_lowercase();

        s1_lower
            .chars()
            .zip(s2_lower.chars())
            .position(|(c1, c2)| c1 != c2)
    }

    /// Suggest collision resolution abbreviations for two conflicting names
    ///
    /// Returns (new_abbrev, existing_abbrev) with additional distinguishing letters
    pub fn suggest_collision_resolution(
        new_name: &str,
        existing_name: &str,
    ) -> Option<(String, String)> {
        // Find first difference position
        let diff_pos = Self::find_first_difference(new_name, existing_name)?;

        // Try to build distinguishing abbreviations
        let new_chars: Vec<char> = new_name.chars().filter(|c| c.is_alphabetic()).collect();
        let existing_chars: Vec<char> = existing_name
            .chars()
            .filter(|c| c.is_alphabetic())
            .collect();

        if new_chars.len() < 4 || existing_chars.len() < 4 {
            return None;
        }

        // Take first 3 chars + the distinguishing char
        let new_abbrev = format!(
            "{}{}",
            new_chars.iter().take(3).collect::<String>(),
            new_chars
                .get(diff_pos.min(new_chars.len() - 1))?
                .to_uppercase()
        )
        .to_uppercase();

        let existing_abbrev = format!(
            "{}{}",
            existing_chars.iter().take(3).collect::<String>(),
            existing_chars
                .get(diff_pos.min(existing_chars.len() - 1))?
                .to_uppercase()
        )
        .to_uppercase();

        Some((new_abbrev, existing_abbrev))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_category_valid() {
        let cat = Category::new("Authentication".to_string(), "AUT".to_string()).unwrap();
        assert_eq!(cat.full_name, "Authentication");
        assert_eq!(cat.abbreviation, "AUT");
    }

    #[test]
    fn test_new_category_converts_to_uppercase() {
        let cat = Category::new("Authentication".to_string(), "aut".to_string()).unwrap();
        assert_eq!(cat.abbreviation, "AUT");
    }

    #[test]
    fn test_new_category_trims_whitespace() {
        let cat = Category::new("  Auth  ".to_string(), "  aut  ".to_string()).unwrap();
        assert_eq!(cat.full_name, "Auth");
        assert_eq!(cat.abbreviation, "AUT");
    }

    #[test]
    fn test_new_category_rejects_empty_name() {
        let result = Category::new("".to_string(), "AUT".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_new_category_rejects_short_abbreviation() {
        let result = Category::new("Auth".to_string(), "AU".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_is_valid_abbreviation() {
        assert!(Category::is_valid_abbreviation("AUT"));
        assert!(Category::is_valid_abbreviation("AUTH"));
        assert!(Category::is_valid_abbreviation("A123"));
        assert!(!Category::is_valid_abbreviation("AU"));
        assert!(!Category::is_valid_abbreviation("A-B"));
        assert!(!Category::is_valid_abbreviation(""));
    }

    #[test]
    fn test_name_matches_case_insensitive() {
        let cat = Category::new("Authentication".to_string(), "AUT".to_string()).unwrap();
        assert!(cat.name_matches("authentication"));
        assert!(cat.name_matches("AUTHENTICATION"));
        assert!(cat.name_matches("  Authentication  "));
        assert!(!cat.name_matches("Authorization"));
    }

    #[test]
    fn test_abbreviation_matches_case_insensitive() {
        let cat = Category::new("Authentication".to_string(), "AUT".to_string()).unwrap();
        assert!(cat.abbreviation_matches("aut"));
        assert!(cat.abbreviation_matches("AUT"));
        assert!(cat.abbreviation_matches("  Aut  "));
        assert!(!cat.abbreviation_matches("AUTH"));
    }

    #[test]
    fn test_suggest_abbreviation() {
        assert_eq!(Category::suggest_abbreviation("Authentication"), "AUT");
        assert_eq!(Category::suggest_abbreviation("authorization"), "AUT");
        assert_eq!(Category::suggest_abbreviation("User Management"), "USE");
        assert_eq!(Category::suggest_abbreviation("API-Gateway"), "API");
        assert_eq!(Category::suggest_abbreviation("ab"), "AB");
    }

    #[test]
    fn test_find_first_difference() {
        assert_eq!(
            Category::find_first_difference("Authentication", "Authorization"),
            Some(4)
        );
        assert_eq!(Category::find_first_difference("Auth", "Auto"), Some(3));
        assert_eq!(Category::find_first_difference("Same", "Same"), None);
        assert_eq!(Category::find_first_difference("AUTH", "auth"), None); // Case insensitive
    }

    #[test]
    fn test_suggest_collision_resolution() {
        let (new_abbrev, existing_abbrev) =
            Category::suggest_collision_resolution("Authentication", "Authorization").unwrap();

        // Both should start with AUT and have different 4th letters
        assert!(new_abbrev.starts_with("AUT"));
        assert!(existing_abbrev.starts_with("AUT"));
        assert_ne!(new_abbrev, existing_abbrev);
        assert_eq!(new_abbrev.len(), 4);
        assert_eq!(existing_abbrev.len(), 4);
    }

    #[test]
    fn test_collision_resolution_short_names() {
        // Should return None for names too short to create 4-char abbreviations
        let result = Category::suggest_collision_resolution("Api", "App");
        assert!(result.is_none());
    }
}
