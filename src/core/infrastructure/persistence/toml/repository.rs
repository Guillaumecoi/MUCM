// TOML-based implementation of UseCaseRepository
use crate::config::Config;
use crate::core::infrastructure::persistence::traits::UseCaseRepository;
use crate::core::{to_snake_case, UseCase};
use anyhow::Result;
use std::fs;
use std::path::Path;

/// Repository implementation that persists use cases to TOML files
///
/// Architecture:
/// - TOML files (.toml) are the source of truth - users edit these
/// - Markdown files (.md) are generated documentation - regenerated from TOML
pub struct TomlUseCaseRepository {
    config: Config,
}

impl TomlUseCaseRepository {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

impl UseCaseRepository for TomlUseCaseRepository {
    fn save(&self, use_case: &UseCase) -> Result<()> {
        self.save_toml_only(use_case)
    }

    fn save_markdown(&self, use_case_id: &str, markdown_content: &str) -> Result<()> {
        self.save_markdown_only(use_case_id, markdown_content)
    }

    fn save_markdown_with_filename(
        &self,
        use_case: &UseCase,
        filename: &str,
        content: &str,
    ) -> Result<()> {
        let category_snake = to_snake_case(&use_case.category);
        // New folder structure: {category}/{use-case-id}/
        let use_case_dir = Path::new(&self.config.directories.use_case_dir)
            .join(&category_snake)
            .join(&use_case.id);
        fs::create_dir_all(&use_case_dir)?;
        let md_path = use_case_dir.join(filename);
        fs::write(&md_path, content)?;
        Ok(())
    }

    fn load_all(&self) -> Result<Vec<UseCase>> {
        let toml_dir = Path::new(&self.config.directories.data_dir);
        let mut use_cases = Vec::new();

        if !toml_dir.exists() {
            return Ok(use_cases); // No use cases yet
        }

        for entry in walkdir::WalkDir::new(toml_dir) {
            let entry = entry?;

            // Only process .toml files that start with "UC-" (use case ID pattern)
            if entry.file_type().is_file()
                && entry.path().extension().is_some_and(|ext| ext == "toml")
                && entry
                    .path()
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().starts_with("UC-"))
            {
                let content = fs::read_to_string(entry.path())?;
                // Parse TOML to intermediate value, then convert to JSON value to ensure
                // extra fields are serde_json::Value instead of toml::Value
                let toml_value: toml::Value = toml::from_str(&content)?;
                let json_str = serde_json::to_string(&toml_value)?;
                let mut use_case: UseCase = serde_json::from_str(&json_str)?;

                // Migration: If use case has no views, add default view
                if use_case.views.is_empty() {
                    use crate::core::MethodologyView;
                    let default_methodology = &self.config.templates.default_methodology;
                    let default_view = MethodologyView::new(default_methodology, "normal");
                    use_case.views.push(default_view);

                    // Auto-save the migrated use case
                    self.save_toml_only(&use_case)?;
                }

                use_cases.push(use_case);
            }
        }

        Ok(use_cases)
    }

    fn load_by_id(&self, id: &str) -> Result<Option<UseCase>> {
        let all_cases = self.load_all()?;
        Ok(all_cases.into_iter().find(|uc| uc.id == id))
    }
}

impl TomlUseCaseRepository {
    fn save_toml_only(&self, use_case: &UseCase) -> Result<()> {
        let category_snake = to_snake_case(&use_case.category);

        // Create TOML directory structure (source files)
        let toml_dir = Path::new(&self.config.directories.data_dir).join(&category_snake);
        fs::create_dir_all(&toml_dir)?;

        // Filter out Null values from extra fields before serialization
        // TOML doesn't support null values like JSON does
        let mut use_case_for_toml = use_case.clone();
        use_case_for_toml.extra.retain(|_, v| !v.is_null());

        // Save TOML file (source of truth)
        let toml_path = toml_dir.join(format!("{}.toml", use_case.id));
        let toml_content = toml::to_string_pretty(&use_case_for_toml)?;
        fs::write(&toml_path, toml_content)?;

        Ok(())
    }

    fn save_markdown_only(&self, use_case_id: &str, markdown_content: &str) -> Result<()> {
        // Load the use case from TOML to get category
        let use_case = self
            .load_by_id(use_case_id)?
            .ok_or_else(|| anyhow::anyhow!("Use case {} not found in TOML", use_case_id))?;

        let category_snake = to_snake_case(&use_case.category);

        // Create markdown directory structure (generated docs) with new nested folder structure
        // New structure: {category}/{use-case-id}/README.md
        let use_case_dir = Path::new(&self.config.directories.use_case_dir)
            .join(&category_snake)
            .join(&use_case.id);
        fs::create_dir_all(&use_case_dir)?;

        // Save markdown file as README.md in use case folder
        let md_path = use_case_dir.join("README.md");
        fs::write(&md_path, markdown_content)?;

        Ok(())
    }
}
