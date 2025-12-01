use crate::core::domain::UseCaseReference;
use crate::core::{UseCase, UseCaseRepository};
use anyhow::Result;

/// Service for managing references between use cases
pub struct ReferenceManagementService<'a> {
    repository: &'a dyn UseCaseRepository,
    use_cases: &'a mut Vec<UseCase>,
}

impl<'a> ReferenceManagementService<'a> {
    pub fn new(repository: &'a dyn UseCaseRepository, use_cases: &'a mut Vec<UseCase>) -> Self {
        Self {
            repository,
            use_cases,
        }
    }

    /// Add a reference to a use case
    pub fn add_reference(
        &mut self,
        use_case_id: &str,
        target_id: String,
        relationship: String,
        description: Option<String>,
    ) -> Result<()> {
        let index = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index].clone();
        let reference = UseCaseReference::new(target_id, relationship);
        let reference = if let Some(desc) = description {
            reference.with_description(desc)
        } else {
            reference
        };
        use_case.add_reference(reference);
        self.repository.save(&use_case)?;
        self.use_cases[index] = use_case;
        Ok(())
    }

    /// Remove a reference from a use case
    pub fn remove_reference(&mut self, use_case_id: &str, target_id: &str) -> Result<()> {
        let index = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index].clone();
        use_case
            .use_case_references
            .retain(|r| r.target_id != target_id);
        self.repository.save(&use_case)?;
        self.use_cases[index] = use_case;
        Ok(())
    }

    // Helper methods
    fn find_use_case_index(&self, use_case_id: &str) -> Result<usize> {
        self.use_cases
            .iter()
            .position(|uc| uc.id == use_case_id)
            .ok_or_else(|| anyhow::anyhow!("Use case '{}' not found", use_case_id))
    }
}
