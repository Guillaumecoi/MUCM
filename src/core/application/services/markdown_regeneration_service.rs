use crate::core::application::generators::{MarkdownGenerator, OutputManager};
use crate::core::utils::suggest_alternatives;
use crate::core::{TemplateEngine, UseCase, UseCaseRepository};
use anyhow::Result;

/// Service for regenerating markdown documentation
///
/// This service handles regeneration of markdown files from TOML source data.
/// It generates markdown for individual use cases.
pub struct MarkdownRegenerationService<'a> {
    repository: &'a dyn UseCaseRepository,
    use_cases: &'a [UseCase],
    markdown_generator: &'a MarkdownGenerator,
    template_engine: &'a TemplateEngine,
}

impl<'a> MarkdownRegenerationService<'a> {
    pub fn new(
        repository: &'a dyn UseCaseRepository,
        use_cases: &'a [UseCase],
        markdown_generator: &'a MarkdownGenerator,
        template_engine: &'a TemplateEngine,
    ) -> Self {
        Self {
            repository,
            use_cases,
            markdown_generator,
            template_engine,
        }
    }

    /// Regenerate use case with different methodology
    pub fn regenerate_use_case_with_methodology(
        &self,
        use_case_id: &str,
        methodology: &str,
    ) -> Result<()> {
        // Find the use case
        let use_case = match self.use_cases.iter().find(|uc| uc.id == use_case_id) {
            Some(uc) => uc.clone(),
            None => {
                // Get available use case IDs for suggestions
                let available_ids: Vec<String> =
                    self.use_cases.iter().map(|uc| uc.id.clone()).collect();
                let error_msg = suggest_alternatives(use_case_id, &available_ids, "Use case");
                return Err(anyhow::anyhow!("{}", error_msg));
            }
        };

        // Validate methodology exists
        let available_methodologies = self.template_engine.available_methodologies();
        if !available_methodologies.contains(&methodology.to_string()) {
            return Err(anyhow::anyhow!(
                "Unknown methodology '{}'. Available: {:?}",
                methodology,
                available_methodologies
            ));
        }

        // Regenerate markdown for all enabled views using OutputManager for consistent naming
        let filenames = OutputManager::generate_all_filenames(&use_case);
        for (filename, view) in filenames {
            let markdown_content =
                self.markdown_generator
                    .generate(&use_case, None, Some(&view))?;
            self.repository
                .save_markdown_with_filename(&use_case, &filename, &markdown_content)?;
        }

        Ok(())
    }

    /// Regenerate markdown for a single use case
    pub fn regenerate_markdown(&self, use_case_id: &str) -> Result<()> {
        // Load use case from TOML (source of truth)
        let use_case = match self.repository.load_by_id(use_case_id)? {
            Some(uc) => uc,
            None => {
                // Get available use case IDs for suggestions
                let available_ids: Vec<String> =
                    self.use_cases.iter().map(|uc| uc.id.clone()).collect();
                let error_msg = suggest_alternatives(use_case_id, &available_ids, "Use case");
                return Err(anyhow::anyhow!("{}", error_msg));
            }
        };

        // Generate markdown for each enabled view using OutputManager for consistent naming
        let filenames = OutputManager::generate_all_filenames(&use_case);
        for (filename, view) in filenames {
            let markdown_content =
                self.markdown_generator
                    .generate(&use_case, None, Some(&view))?;
            self.repository
                .save_markdown_with_filename(&use_case, &filename, &markdown_content)?;
        }

        Ok(())
    }
}
