use crate::core::{UseCase, UseCaseRepository};
use anyhow::Result;

/// Service for managing preconditions and postconditions
pub struct PreconditionPostconditionService<'a> {
    repository: &'a dyn UseCaseRepository,
    use_cases: &'a mut Vec<UseCase>,
}

impl<'a> PreconditionPostconditionService<'a> {
    pub fn new(
        repository: &'a dyn UseCaseRepository,
        use_cases: &'a mut Vec<UseCase>,
    ) -> Self {
        Self {
            repository,
            use_cases,
        }
    }

    /// Add a precondition to a use case
    pub fn add_precondition(&mut self, use_case_id: &str, precondition: String) -> Result<()> {
        let index = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index].clone();
        use_case.add_precondition(precondition.into());
        self.repository.save(&use_case)?;
        self.use_cases[index] = use_case;
        Ok(())
    }

    /// Remove a precondition from a use case
    pub fn remove_precondition(&mut self, use_case_id: &str, index: usize) -> Result<()> {
        let index_in_vec = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index_in_vec].clone();

        // Convert 1-based index to 0-based
        let zero_based_index = index.saturating_sub(1);
        if zero_based_index >= use_case.preconditions.len() {
            return Err(anyhow::anyhow!(
                "Precondition index {} is out of bounds",
                index
            ));
        }

        use_case.preconditions.remove(zero_based_index);
        self.repository.save(&use_case)?;
        self.use_cases[index_in_vec] = use_case;
        Ok(())
    }

    /// Add a postcondition to a use case
    pub fn add_postcondition(&mut self, use_case_id: &str, postcondition: String) -> Result<()> {
        let index = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index].clone();
        use_case.add_postcondition(postcondition.into());
        self.repository.save(&use_case)?;
        self.use_cases[index] = use_case;
        Ok(())
    }

    /// Remove a postcondition from a use case
    pub fn remove_postcondition(&mut self, use_case_id: &str, index: usize) -> Result<()> {
        let index_in_vec = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index_in_vec].clone();

        // Convert 1-based index to 0-based
        let zero_based_index = index.saturating_sub(1);
        if zero_based_index >= use_case.postconditions.len() {
            return Err(anyhow::anyhow!(
                "Postcondition index {} is out of bounds",
                index
            ));
        }

        use_case.postconditions.remove(zero_based_index);
        self.repository.save(&use_case)?;
        self.use_cases[index_in_vec] = use_case;
        Ok(())
    }

    /// Edit a precondition in a use case
    pub fn edit_precondition(
        &mut self,
        use_case_id: &str,
        index: usize,
        new_text: String,
    ) -> Result<()> {
        let index_in_vec = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index_in_vec].clone();

        // Convert 1-based index to 0-based
        let zero_based_index = index.saturating_sub(1);
        if zero_based_index >= use_case.preconditions.len() {
            return Err(anyhow::anyhow!(
                "Precondition index {} is out of bounds",
                index
            ));
        }

        use_case.preconditions[zero_based_index] = new_text.into();
        self.repository.save(&use_case)?;
        self.use_cases[index_in_vec] = use_case;
        Ok(())
    }

    /// Edit a postcondition in a use case
    pub fn edit_postcondition(
        &mut self,
        use_case_id: &str,
        index: usize,
        new_text: String,
    ) -> Result<()> {
        let index_in_vec = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index_in_vec].clone();

        // Convert 1-based index to 0-based
        let zero_based_index = index.saturating_sub(1);
        if zero_based_index >= use_case.postconditions.len() {
            return Err(anyhow::anyhow!(
                "Postcondition index {} is out of bounds",
                index
            ));
        }

        use_case.postconditions[zero_based_index] = new_text.into();
        self.repository.save(&use_case)?;
        self.use_cases[index_in_vec] = use_case;
        Ok(())
    }

    /// Reorder preconditions in a use case
    pub fn reorder_preconditions(
        &mut self,
        use_case_id: &str,
        from_index: usize,
        to_index: usize,
    ) -> Result<()> {
        let index_in_vec = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index_in_vec].clone();

        // Convert 1-based indices to 0-based
        let from_idx = from_index.saturating_sub(1);
        let to_idx = to_index.saturating_sub(1);

        if from_idx >= use_case.preconditions.len() || to_idx >= use_case.preconditions.len() {
            return Err(anyhow::anyhow!("Index out of bounds"));
        }

        // Remove item from old position
        let item = use_case.preconditions.remove(from_idx);
        // Insert at new position
        use_case.preconditions.insert(to_idx, item);

        self.repository.save(&use_case)?;
        self.use_cases[index_in_vec] = use_case;
        Ok(())
    }

    /// Reorder postconditions in a use case
    pub fn reorder_postconditions(
        &mut self,
        use_case_id: &str,
        from_index: usize,
        to_index: usize,
    ) -> Result<()> {
        let index_in_vec = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index_in_vec].clone();

        // Convert 1-based indices to 0-based
        let from_idx = from_index.saturating_sub(1);
        let to_idx = to_index.saturating_sub(1);

        if from_idx >= use_case.postconditions.len() || to_idx >= use_case.postconditions.len() {
            return Err(anyhow::anyhow!("Index out of bounds"));
        }

        // Remove item from old position
        let item = use_case.postconditions.remove(from_idx);
        // Insert at new position
        use_case.postconditions.insert(to_idx, item);

        self.repository.save(&use_case)?;
        self.use_cases[index_in_vec] = use_case;
        Ok(())
    }

    /// Clear all preconditions from a use case
    pub fn clear_preconditions(&mut self, use_case_id: &str) -> Result<()> {
        let index_in_vec = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index_in_vec].clone();

        use_case.preconditions.clear();
        self.repository.save(&use_case)?;
        self.use_cases[index_in_vec] = use_case;
        Ok(())
    }

    /// Clear all postconditions from a use case
    pub fn clear_postconditions(&mut self, use_case_id: &str) -> Result<()> {
        let index_in_vec = self.find_use_case_index(use_case_id)?;
        let mut use_case = self.use_cases[index_in_vec].clone();

        use_case.postconditions.clear();
        self.repository.save(&use_case)?;
        self.use_cases[index_in_vec] = use_case;
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
