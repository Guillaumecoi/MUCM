//! Output manager for generating filenames for use case documentation.
//!
//! Handles filename generation logic for both single-view and multi-view use cases:
//! - Single view: `UC-001.md` (no suffix)
//! - Multiple views: `UC-001-feat-s.md`, `UC-001-bus-n.md` (with methodology-level suffix)

use crate::core::{MethodologyView, UseCase};

/// Manages output filenames for use case documentation.
pub struct OutputManager;

impl OutputManager {
    /// Generates all filenames for a use case based on its enabled views.
    ///
    /// Returns a vector of (filename, view) tuples for each enabled view.
    /// Every use case must have at least one view.
    pub fn generate_all_filenames(use_case: &UseCase) -> Vec<(String, MethodologyView)> {
        use_case
            .enabled_views()
            .map(|view| {
                let filename = format!("{}-{}.md", use_case.id, view.key());
                (filename, view.clone())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{MethodologyView, UseCase};

    #[test]
    fn test_generate_all_filenames_single_view() {
        let mut use_case = UseCase::new_with_auto_abbreviation(
            "UC-001".to_string(),
            "Test Use Case".to_string(),
            "testing".to_string(),
            "Description".to_string(),
            "medium".to_string(),
        )
        .unwrap();

        // Add a single view
        use_case.add_view(MethodologyView::new(
            "business".to_string(),
            "normal".to_string(),
        ));

        let filenames = OutputManager::generate_all_filenames(&use_case);

        assert_eq!(filenames.len(), 1);
        assert_eq!(filenames[0].0, "UC-001-business-normal.md");
        assert_eq!(filenames[0].1.methodology, "business");
        assert_eq!(filenames[0].1.level, "normal");
    }

    #[test]
    fn test_generate_all_filenames_multi_view() {
        let mut use_case = UseCase::new_with_auto_abbreviation(
            "UC-001".to_string(),
            "Test Use Case".to_string(),
            "testing".to_string(),
            "Description".to_string(),
            "medium".to_string(),
        )
        .unwrap();

        use_case.add_view(MethodologyView::new(
            "feature".to_string(),
            "simple".to_string(),
        ));
        use_case.add_view(MethodologyView::new(
            "business".to_string(),
            "normal".to_string(),
        ));

        let filenames = OutputManager::generate_all_filenames(&use_case);

        assert_eq!(filenames.len(), 2);

        // Check that both filenames are present (order may vary)
        let filename_strings: Vec<String> = filenames.iter().map(|(f, _)| f.clone()).collect();
        assert!(filename_strings.contains(&"UC-001-feature-simple.md".to_string()));
        assert!(filename_strings.contains(&"UC-001-business-normal.md".to_string()));

        // All should have associated views
        assert_eq!(filenames[0].1.methodology, "feature");
        assert_eq!(filenames[1].1.methodology, "business");
    }

    #[test]
    fn test_generate_all_filenames_with_disabled_view() {
        let mut use_case = UseCase::new_with_auto_abbreviation(
            "UC-001".to_string(),
            "Test Use Case".to_string(),
            "testing".to_string(),
            "Description".to_string(),
            "medium".to_string(),
        )
        .unwrap();

        use_case.add_view(MethodologyView::new(
            "feature".to_string(),
            "simple".to_string(),
        ));

        let mut disabled_view = MethodologyView::new("business".to_string(), "normal".to_string());
        disabled_view.enabled = false;
        use_case.add_view(disabled_view);

        let filenames = OutputManager::generate_all_filenames(&use_case);

        // Should only include the enabled view
        assert_eq!(filenames.len(), 1);
        assert_eq!(filenames[0].0, "UC-001-feature-simple.md");
    }
}
