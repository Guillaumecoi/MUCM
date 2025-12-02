use crate::config::Config;
use crate::core::application::MethodologyFieldCollector;
use crate::core::domain::UseCaseService;
use crate::core::{MethodologyView, UseCase, UseCaseRepository};
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

/// Parameters for creating a use case
pub struct UseCaseCreationParams {
    pub title: String,
    pub category: String,
    pub category_abbreviation: String,
    pub description: Option<String>,
    pub priority: String,
    pub existing_use_cases: Vec<UseCase>,
}

/// Extended parameters with methodology configuration
pub struct UseCaseWithMethodologyParams {
    pub base: UseCaseCreationParams,
    pub methodology: String,
}

/// Extended parameters with custom fields
pub struct UseCaseWithCustomFieldsParams {
    pub base: UseCaseCreationParams,
    pub methodology: String,
    pub user_fields: HashMap<String, String>,
}

/// Extended parameters with multiple views
pub struct UseCaseWithViewsParams {
    pub base: UseCaseCreationParams,
    pub views: Vec<(String, String)>,
    pub extra_fields: HashMap<String, String>,
}

/// Handles use case creation with methodology support
pub struct UseCaseCreator {
    config: Config,
    use_case_service: UseCaseService,
}

impl UseCaseCreator {
    pub fn new(config: Config) -> Self {
        Self {
            use_case_service: UseCaseService::new(),
            config,
        }
    }

    /// Create a use case with methodology-specific custom fields
    pub fn create_use_case_with_methodology(
        &self,
        params: UseCaseWithMethodologyParams,
        repository: &dyn UseCaseRepository,
    ) -> Result<UseCase> {
        let use_case_id = self.use_case_service.generate_unique_use_case_id(
            &params.base.category,
            &params.base.category_abbreviation,
            &params.base.existing_use_cases,
            &self.config.directories.use_case_dir,
        );
        let description = params.base.description.unwrap_or_default();

        // Create base use case with explicit abbreviation
        let mut use_case = UseCase::new(
            use_case_id.clone(),
            params.base.title,
            params.base.category.clone(),
            params.base.category_abbreviation.clone(),
            description,
            params.base.priority,
        )
        .map_err(|e: String| anyhow::anyhow!(e))?;

        // Add default view (methodology:normal)
        use_case.add_view(MethodologyView::new(
            params.methodology.clone(),
            "normal".to_string(),
        ));

        // Collect and store methodology fields for this view
        let collector = MethodologyFieldCollector::new()?;
        let collection = collector
            .collect_fields_for_views(&[(params.methodology.clone(), "normal".to_string())])?;

        // Store fields grouped by methodology
        if !collection.fields.is_empty() {
            let mut methodology_fields = HashMap::new();
            for (field_name, field_config) in collection.fields {
                let value = field_config
                    .default
                    .map(serde_json::Value::String)
                    .unwrap_or(serde_json::Value::Null);
                methodology_fields.insert(field_name, value);
            }
            use_case
                .methodology_fields
                .insert(params.methodology.clone(), methodology_fields);
        }

        // Step 1: Save TOML first (source of truth)
        repository.save(&use_case)?;

        // Step 2: Load from TOML to ensure we're working with persisted data
        let use_case_from_toml = repository
            .load_by_id(&use_case.id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to load newly created use case from TOML"))?;

        Ok(use_case_from_toml)
    }

    pub fn create_use_case_with_custom_fields(
        &self,
        params: UseCaseWithCustomFieldsParams,
        repository: &dyn UseCaseRepository,
    ) -> Result<UseCase> {
        let use_case_id = self.use_case_service.generate_unique_use_case_id(
            &params.base.category,
            &params.base.category_abbreviation,
            &params.base.existing_use_cases,
            &self.config.directories.use_case_dir,
        );
        let description = params.base.description.unwrap_or_default();

        // Create base use case with explicit abbreviation
        let mut use_case = UseCase::new(
            use_case_id.clone(),
            params.base.title,
            params.base.category.clone(),
            params.base.category_abbreviation.clone(),
            description,
            params.base.priority,
        )
        .map_err(|e: String| anyhow::anyhow!(e))?;

        // Add default view (methodology:normal)
        use_case.add_view(MethodologyView::new(
            params.methodology.clone(),
            "normal".to_string(),
        ));

        // Collect methodology fields
        let collector = MethodologyFieldCollector::new()?;
        let collection = collector
            .collect_fields_for_views(&[(params.methodology.clone(), "normal".to_string())])?;

        // Store fields grouped by methodology, with user overrides
        let mut methodology_fields = HashMap::new();
        for (field_name, field_config) in collection.fields {
            // Use user-provided value if available, otherwise use default
            let value = if let Some(user_value) = params.user_fields.get(&field_name) {
                serde_json::Value::String(user_value.clone())
            } else if let Some(default) = field_config.default {
                serde_json::Value::String(default)
            } else {
                serde_json::Value::Null
            };
            methodology_fields.insert(field_name, value);
        }

        // Add any user fields that weren't in the methodology definition
        for (key, value) in params.user_fields {
            methodology_fields
                .entry(key)
                .or_insert(serde_json::Value::String(value));
        }

        if !methodology_fields.is_empty() {
            use_case
                .methodology_fields
                .insert(params.methodology.clone(), methodology_fields);
        }

        // Step 1: Save TOML first (source of truth)
        repository.save(&use_case)?;

        // Step 2: Load from TOML to ensure we're working with persisted data
        let use_case_from_toml = repository
            .load_by_id(&use_case.id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to load newly created use case from TOML"))?;

        Ok(use_case_from_toml)
    }

    /// Create a use case with multiple methodology views and collected fields
    ///
    /// This method properly uses MethodologyFieldCollector to gather fields from all views,
    /// stores them in methodology_fields structure, and handles user value overrides.
    pub fn create_use_case_with_views(
        &self,
        params: UseCaseWithViewsParams,
        repository: &dyn UseCaseRepository,
    ) -> Result<UseCase> {
        let use_case_id = self.use_case_service.generate_unique_use_case_id(
            &params.base.category,
            &params.base.category_abbreviation,
            &params.base.existing_use_cases,
            &self.config.directories.use_case_dir,
        );
        let description = params.base.description.unwrap_or_default();

        // Convert view tuples to MethodologyView objects
        let views: Vec<MethodologyView> = params
            .views
            .iter()
            .map(|(methodology, level)| MethodologyView::new(methodology.clone(), level.clone()))
            .collect();

        // Collect fields from all methodology views using the collector
        // If collector fails (e.g., in test environment without methodologies), use empty fields
        let collector = MethodologyFieldCollector::new()?;
        let view_pairs: Vec<(String, String)> = views
            .iter()
            .map(|v| (v.methodology.clone(), v.level.clone()))
            .collect();

        let field_collection = match collector.collect_fields_for_views(&view_pairs) {
            Ok(collection) => collection,
            Err(e) => {
                eprintln!(
                    "Warning: Could not collect methodology fields: {}. Using empty fields.",
                    e
                );
                Default::default()
            }
        };

        // Display any warnings (e.g., standard field conflicts)
        for warning in &field_collection.warnings {
            eprintln!("{}", warning);
        }

        // Apply user-provided values to the collected fields
        let methodology_field_values =
            collector.apply_user_values(&field_collection, params.extra_fields);

        // Group fields by methodology for storage in methodology_fields
        let mut methodology_fields: HashMap<String, HashMap<String, Value>> = HashMap::new();

        // Initialize empty HashMap for each methodology in views
        for view in &views {
            methodology_fields
                .entry(view.methodology.clone())
                .or_default();
        }

        // Populate with actual field values
        for (field_name, field_value) in methodology_field_values {
            // Find which methodology this field belongs to
            if let Some(collected_field) = field_collection.fields.get(&field_name) {
                for methodology in &collected_field.methodologies {
                    methodology_fields
                        .entry(methodology.clone())
                        .or_default()
                        .insert(field_name.clone(), field_value.clone());
                }
            }
        }

        // Create the use case with explicit abbreviation from parameter
        let mut use_case = UseCase::new(
            use_case_id.clone(),
            params.base.title,
            params.base.category.clone(),
            params.base.category_abbreviation.clone(),
            description,
            params.base.priority,
        )
        .map_err(|e| anyhow::anyhow!(e))?;

        // Set methodology fields
        use_case.methodology_fields = methodology_fields;

        // Add all views
        for view in views {
            use_case.add_view(view);
        }

        // Save and reload from TOML
        repository.save(&use_case)?;
        let use_case_from_toml = repository
            .load_by_id(&use_case.id)?
            .ok_or_else(|| anyhow::anyhow!("Failed to load newly created use case from TOML"))?;

        Ok(use_case_from_toml)
    }
}
