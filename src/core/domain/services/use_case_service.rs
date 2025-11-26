// Domain service for use case business logic
use crate::core::domain::UseCase;
use crate::core::to_snake_case;
use std::path::Path;

/// Core business logic for use case management
/// This service focuses purely on domain operations without I/O concerns
#[derive(Clone)]
pub struct UseCaseService;

impl UseCaseService {
    pub fn new() -> Self {
        Self
    }

    /// Generate a use case ID based on category abbreviation and existing use cases
    /// Checks both in-memory use cases and filesystem for unique sequential numbering
    ///
    /// # Arguments
    /// * `category` - Full category name (for filesystem path generation)
    /// * `category_abbreviation` - Category abbreviation to use in ID (e.g., "AUT")
    /// * `use_cases` - Existing use cases to check for ID conflicts
    /// * `use_case_dir` - Root directory for use case markdown files
    ///
    /// # Returns
    /// Unique use case ID in format "UC-{ABBR}-{NNN}" (e.g., "UC-AUT-001")
    pub fn generate_unique_use_case_id(
        &self,
        category: &str,
        category_abbreviation: &str,
        use_cases: &[UseCase],
        use_case_dir: &str,
    ) -> String {
        let category_prefix = category_abbreviation.to_uppercase();
        let category_dir = Path::new(use_case_dir).join(to_snake_case(category));

        // Find the highest existing number by checking both in-memory and filesystem
        let mut max_number = 0;

        // Check in-memory use cases
        for uc in use_cases.iter() {
            if uc.category.to_lowercase() == category.to_lowercase() {
                if let Some(num) = self.extract_number_from_id(&uc.id, &category_prefix) {
                    max_number = max_number.max(num);
                }
            }
        }

        // Check filesystem for existing files
        if category_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&category_dir) {
                for entry in entries.flatten() {
                    if let Some(file_name) = entry.file_name().to_str() {
                        if file_name.ends_with(".md")
                            && file_name.starts_with(&format!("UC-{}-", category_prefix))
                        {
                            let id_part = file_name.trim_end_matches(".md");
                            if let Some(num) =
                                self.extract_number_from_id(id_part, &category_prefix)
                            {
                                max_number = max_number.max(num);
                            }
                        }
                    }
                }
            }
        }

        format!("UC-{}-{:03}", category_prefix, max_number + 1)
    }

    /// Extract number from ID like "UC-CON-001" -> Some(1)
    fn extract_number_from_id(&self, id: &str, category_prefix: &str) -> Option<usize> {
        let expected_prefix = format!("UC-{}-", category_prefix);
        if id.starts_with(&expected_prefix) {
            let number_part = &id[expected_prefix.len()..];
            number_part.parse::<usize>().ok()
        } else {
            None
        }
    }
}

impl Default for UseCaseService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_use_case(
        id: String,
        title: String,
        category: String,
        description: String,
    ) -> UseCase {
        UseCase::new_with_auto_abbreviation(id, title, category, description, "Medium".to_string())
            .unwrap()
    }

    fn find_use_case_by_id<'a>(use_cases: &'a [UseCase], id: &str) -> Option<&'a UseCase> {
        use_cases.iter().find(|uc| uc.id == id)
    }

    #[test]
    fn test_use_case_service_unique_id_generation() {
        let service = UseCaseService::new();

        // Test unique ID generation with filesystem checks
        let existing_use_cases = vec![
            create_test_use_case(
                "UC-SEC-001".to_string(),
                "Login".to_string(),
                "Security".to_string(),
                "".to_string(),
            ),
            create_test_use_case(
                "UC-API-001".to_string(),
                "REST API".to_string(),
                "API".to_string(),
                "".to_string(),
            ),
        ];

        // Use a temporary directory for testing
        let temp_dir = std::env::temp_dir().join("mucm_test_use_case_service");
        let temp_dir_str = temp_dir.to_string_lossy();

        let new_id = service.generate_unique_use_case_id(
            "Security",
            "SEC",
            &existing_use_cases,
            &temp_dir_str,
        );
        assert!(new_id.starts_with("UC-SEC-"));
        assert!(new_id.len() > 7); // Should have format UC-SEC-XXX

        let api_id =
            service.generate_unique_use_case_id("API", "API", &existing_use_cases, &temp_dir_str);
        assert!(api_id.starts_with("UC-API-"));

        let new_category_id = service.generate_unique_use_case_id(
            "Database",
            "DAT",
            &existing_use_cases,
            &temp_dir_str,
        );
        assert!(new_category_id.starts_with("UC-DAT-"));
    }

    #[test]
    fn test_finding_use_cases() {
        let use_cases = vec![
            create_test_use_case(
                "UC-SEC-001".to_string(),
                "Login".to_string(),
                "Security".to_string(),
                "".to_string(),
            ),
            create_test_use_case(
                "UC-API-001".to_string(),
                "REST API".to_string(),
                "API".to_string(),
                "".to_string(),
            ),
        ];

        let found = find_use_case_by_id(&use_cases, "UC-SEC-001");
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Login");

        let not_found = find_use_case_by_id(&use_cases, "UC-MISSING-001");
        assert!(not_found.is_none());
    }
}
