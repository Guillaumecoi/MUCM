// src/core/utils/fuzzy_match.rs
/// Utilities for fuzzy matching and helpful error messages
use std::cmp::min;

/// Calculate Levenshtein distance between two strings
/// Used for "did you mean?" suggestions when user enters invalid use case IDs
pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    // Initialize first column
    for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) {
        row[0] = i;
    }
    // Initialize first row
    for (j, cell) in matrix[0].iter_mut().enumerate().take(len2 + 1) {
        *cell = j;
    }

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1.chars().nth(i - 1) == s2.chars().nth(j - 1) {
                0
            } else {
                1
            };

            matrix[i][j] = min(
                min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                matrix[i - 1][j - 1] + cost,
            );
        }
    }

    matrix[len1][len2]
}

/// Find closest matches from a list of options
/// Used in CLI commands when user types invalid category or use case ID
pub fn find_closest_matches(input: &str, options: &[String], max_distance: usize) -> Vec<String> {
    let mut matches: Vec<(String, usize)> = options
        .iter()
        .map(|opt| {
            let distance = levenshtein_distance(input, opt);
            (opt.clone(), distance)
        })
        .filter(|(_, dist)| *dist <= max_distance)
        .collect();

    matches.sort_by_key(|(_, dist)| *dist);
    matches.into_iter().take(3).map(|(opt, _)| opt).collect()
}

/// Create a helpful error message with suggestions
/// Integrated into error handling when use case IDs are not found
pub fn suggest_alternatives(input: &str, available: &[String], item_type: &str) -> String {
    let suggestions = find_closest_matches(input, available, 3);

    if suggestions.is_empty() {
        format!(
            "{} '{}' not found. Use 'mucm list' to see all available {}s.",
            item_type,
            input,
            item_type.to_lowercase()
        )
    } else {
        format!(
            "{} '{}' not found. Did you mean:\n  {}\n\nUse 'mucm list' to see all available {}s.",
            item_type,
            input,
            suggestions.join("\n  "),
            item_type.to_lowercase()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("saturday", "sunday"), 3);
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(levenshtein_distance("abc", "abc"), 0);
    }

    #[test]
    fn test_find_closest_matches() {
        let options = vec![
            "UC-AUT-001".to_string(),
            "UC-AUT-002".to_string(),
            "UC-PAY-001".to_string(),
        ];

        // "UC-AUTHENTICATION-001" vs "UC-AUT-001" has distance of 13 (too far)
        // Use a closer typo instead
        let matches = find_closest_matches("UC-AUT-001", &options, 3);
        assert!(!matches.is_empty());
        assert_eq!(matches[0], "UC-AUT-001");

        // Test with actual typo
        let matches = find_closest_matches("UC-AUTH-001", &options, 3);
        assert!(!matches.is_empty());
        assert_eq!(matches[0], "UC-AUT-001");
    }

    #[test]
    fn test_suggest_alternatives() {
        let available = vec!["UC-AUT-001".to_string(), "UC-PAY-001".to_string()];

        let suggestion = suggest_alternatives("UC-AUTH-001", &available, "Use case");
        assert!(suggestion.contains("Did you mean"));
        assert!(suggestion.contains("UC-AUT-001"));
    }
}
