use crate::core::{UseCase, UseCaseRepository};
use anyhow::Result;
use std::collections::HashSet;

/// Service for cleaning up orphaned methodology fields
pub struct MethodologyFieldCleanupService<'a> {
    repository: &'a dyn UseCaseRepository,
    use_cases: &'a mut Vec<UseCase>,
}

impl<'a> MethodologyFieldCleanupService<'a> {
    pub fn new(
        repository: &'a dyn UseCaseRepository,
        use_cases: &'a mut Vec<UseCase>,
    ) -> Self {
        Self {
            repository,
            use_cases,
        }
    }

    /// Clean up orphaned methodology fields from use cases
    ///
    /// Scans methodology_fields HashMap in each use case and removes entries for
    /// methodologies that are not currently used by any enabled view.
    pub fn cleanup_methodology_fields(
        &mut self,
        use_case_id: Option<String>,
        dry_run: bool,
    ) -> Result<(usize, usize, Vec<(String, Vec<String>)>)> {
        let mut cleaned_count = 0;
        let mut total_checked = 0;
        let mut details = Vec::new();

        let use_case_ids: Vec<String> = if let Some(id) = use_case_id {
            if !self.use_cases.iter().any(|uc| uc.id == id) {
                anyhow::bail!("Use case '{}' not found", id);
            }
            vec![id]
        } else {
            self.use_cases.iter().map(|uc| uc.id.clone()).collect()
        };

        for uc_id in use_case_ids {
            total_checked += 1;
            let index = self.find_use_case_index(&uc_id)?;
            let use_case = &mut self.use_cases[index];

            let active_methodologies: HashSet<String> = use_case
                .enabled_views()
                .map(|v| v.methodology.clone())
                .collect();

            let orphaned: Vec<String> = use_case
                .methodology_fields
                .keys()
                .filter(|m| !active_methodologies.contains(*m))
                .cloned()
                .collect();

            if !orphaned.is_empty() {
                cleaned_count += 1;
                details.push((uc_id.clone(), orphaned.clone()));

                if !dry_run {
                    for methodology in &orphaned {
                        use_case.methodology_fields.remove(methodology);
                    }
                    self.repository.save(use_case)?;
                }
            }
        }

        Ok((cleaned_count, total_checked, details))
    }

    // Helper methods
    fn find_use_case_index(&self, use_case_id: &str) -> Result<usize> {
        self.use_cases
            .iter()
            .position(|uc| uc.id == use_case_id)
            .ok_or_else(|| anyhow::anyhow!("Use case '{}' not found", use_case_id))
    }
}
