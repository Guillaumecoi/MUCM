//! TOML-based persistence for categories.
//!
//! Stores categories in .config/.mucm/categories.toml with both full names and abbreviations.

use crate::core::domain::Category;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// TOML storage format for categories
#[derive(Debug, Serialize, Deserialize)]
struct CategoriesFile {
    #[serde(default)]
    categories: Vec<CategoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CategoryEntry {
    full_name: String,
    abbreviation: String,
}

/// Repository for managing category persistence
pub struct TomlCategoryRepository {
    config_dir: String,
}

impl TomlCategoryRepository {
    const CATEGORIES_FILE: &'static str = "categories.toml";

    /// Create a new repository with the given config directory
    pub fn new(config_dir: String) -> Self {
        Self { config_dir }
    }

    /// Get the full path to the categories file
    fn get_categories_path(&self) -> std::path::PathBuf {
        Path::new(&self.config_dir).join(Self::CATEGORIES_FILE)
    }

    /// Load all categories from the TOML file
    pub fn load_all(&self) -> Result<Vec<Category>> {
        let path = self.get_categories_path();

        if !path.exists() {
            return Ok(Vec::new());
        }

        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read categories from {:?}", path))?;

        let file: CategoriesFile = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse categories TOML from {:?}", path))?;

        file.categories
            .into_iter()
            .map(|entry| Category::new(entry.full_name, entry.abbreviation))
            .collect()
    }

    /// Save all categories to the TOML file
    pub fn save_all(&self, categories: &[Category]) -> Result<()> {
        let path = self.get_categories_path();

        // Ensure config directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory {:?}", parent))?;
        }

        let entries: Vec<CategoryEntry> = categories
            .iter()
            .map(|cat| CategoryEntry {
                full_name: cat.full_name.clone(),
                abbreviation: cat.abbreviation.clone(),
            })
            .collect();

        let file = CategoriesFile {
            categories: entries,
        };

        let toml_string =
            toml::to_string_pretty(&file).context("Failed to serialize categories to TOML")?;

        fs::write(&path, toml_string)
            .with_context(|| format!("Failed to write categories to {:?}", path))?;

        Ok(())
    }

    /// Check if a category name exists (case-insensitive)
    pub fn exists_name(&self, name: &str) -> Result<bool> {
        let categories = self.load_all()?;
        Ok(categories.iter().any(|cat| cat.name_matches(name)))
    }

    /// Check if a category abbreviation exists (case-insensitive)
    pub fn exists_abbreviation(&self, abbreviation: &str) -> Result<bool> {
        let categories = self.load_all()?;
        Ok(categories
            .iter()
            .any(|cat| cat.abbreviation_matches(abbreviation)))
    }

    /// Find a category by name (case-insensitive)
    pub fn find_by_name(&self, name: &str) -> Result<Option<Category>> {
        let categories = self.load_all()?;
        Ok(categories.into_iter().find(|cat| cat.name_matches(name)))
    }

    /// Find a category by abbreviation (case-insensitive)
    pub fn find_by_abbreviation(&self, abbreviation: &str) -> Result<Option<Category>> {
        let categories = self.load_all()?;
        Ok(categories
            .into_iter()
            .find(|cat| cat.abbreviation_matches(abbreviation)))
    }

    /// Add a new category
    pub fn add(&self, category: Category) -> Result<()> {
        let mut categories = self.load_all()?;

        // Check for duplicates
        if categories
            .iter()
            .any(|cat| cat.name_matches(&category.full_name))
        {
            anyhow::bail!("Category with name '{}' already exists", category.full_name);
        }

        if categories
            .iter()
            .any(|cat| cat.abbreviation_matches(&category.abbreviation))
        {
            anyhow::bail!(
                "Category with abbreviation '{}' already exists",
                category.abbreviation
            );
        }

        categories.push(category);
        self.save_all(&categories)?;

        Ok(())
    }

    /// Update an existing category's abbreviation
    pub fn update_abbreviation(
        &self,
        old_abbreviation: &str,
        new_abbreviation: &str,
    ) -> Result<()> {
        let mut categories = self.load_all()?;

        // Check if new abbreviation is already in use by another category
        if categories.iter().any(|c| {
            c.abbreviation_matches(new_abbreviation) && !c.abbreviation_matches(old_abbreviation)
        }) {
            anyhow::bail!(
                "Abbreviation '{}' is already in use by another category",
                new_abbreviation
            );
        }

        // Find and update the category
        let cat = categories
            .iter_mut()
            .find(|cat| cat.abbreviation_matches(old_abbreviation))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Category with abbreviation '{}' not found",
                    old_abbreviation
                )
            })?;

        cat.abbreviation = new_abbreviation.to_uppercase();
        self.save_all(&categories)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_repo() -> (TomlCategoryRepository, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().to_str().unwrap().to_string();
        (TomlCategoryRepository::new(config_dir), temp_dir)
    }

    #[test]
    fn test_load_all_empty() {
        let (repo, _temp) = create_test_repo();
        let categories = repo.load_all().unwrap();
        assert_eq!(categories.len(), 0);
    }

    #[test]
    fn test_add_and_load() {
        let (repo, _temp) = create_test_repo();

        let cat = Category::new("Authentication".to_string(), "AUT".to_string()).unwrap();
        repo.add(cat.clone()).unwrap();

        let loaded = repo.load_all().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].full_name, "Authentication");
        assert_eq!(loaded[0].abbreviation, "AUT");
    }

    #[test]
    fn test_add_duplicate_name_fails() {
        let (repo, _temp) = create_test_repo();

        let cat1 = Category::new("Authentication".to_string(), "AUT".to_string()).unwrap();
        repo.add(cat1).unwrap();

        let cat2 = Category::new("AUTHENTICATION".to_string(), "AUTH".to_string()).unwrap();
        let result = repo.add(cat2);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_duplicate_abbreviation_fails() {
        let (repo, _temp) = create_test_repo();

        let cat1 = Category::new("Authentication".to_string(), "AUT".to_string()).unwrap();
        repo.add(cat1).unwrap();

        let cat2 = Category::new("Authorization".to_string(), "aut".to_string()).unwrap();
        let result = repo.add(cat2);
        assert!(result.is_err());
    }

    #[test]
    fn test_exists_name_case_insensitive() {
        let (repo, _temp) = create_test_repo();

        let cat = Category::new("Authentication".to_string(), "AUT".to_string()).unwrap();
        repo.add(cat).unwrap();

        assert!(repo.exists_name("Authentication").unwrap());
        assert!(repo.exists_name("authentication").unwrap());
        assert!(repo.exists_name("AUTHENTICATION").unwrap());
        assert!(!repo.exists_name("Authorization").unwrap());
    }

    #[test]
    fn test_exists_abbreviation_case_insensitive() {
        let (repo, _temp) = create_test_repo();

        let cat = Category::new("Authentication".to_string(), "AUT".to_string()).unwrap();
        repo.add(cat).unwrap();

        assert!(repo.exists_abbreviation("AUT").unwrap());
        assert!(repo.exists_abbreviation("aut").unwrap());
        assert!(repo.exists_abbreviation("Aut").unwrap());
        assert!(!repo.exists_abbreviation("AUTH").unwrap());
    }

    #[test]
    fn test_find_by_name() {
        let (repo, _temp) = create_test_repo();

        let cat = Category::new("Authentication".to_string(), "AUT".to_string()).unwrap();
        repo.add(cat).unwrap();

        let found = repo.find_by_name("authentication").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().abbreviation, "AUT");

        let not_found = repo.find_by_name("Authorization").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_find_by_abbreviation() {
        let (repo, _temp) = create_test_repo();

        let cat = Category::new("Authentication".to_string(), "AUT".to_string()).unwrap();
        repo.add(cat).unwrap();

        let found = repo.find_by_abbreviation("aut").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().full_name, "Authentication");

        let not_found = repo.find_by_abbreviation("AUTH").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_update_abbreviation() {
        let (repo, _temp) = create_test_repo();

        let cat = Category::new("Authentication".to_string(), "AUT".to_string()).unwrap();
        repo.add(cat).unwrap();

        repo.update_abbreviation("AUT", "AUTH").unwrap();

        let found = repo.find_by_abbreviation("AUTH").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().full_name, "Authentication");

        let old = repo.find_by_abbreviation("AUT").unwrap();
        assert!(old.is_none());
    }

    #[test]
    fn test_update_abbreviation_collision_fails() {
        let (repo, _temp) = create_test_repo();

        let cat1 = Category::new("Authentication".to_string(), "AUT".to_string()).unwrap();
        let cat2 = Category::new("Authorization".to_string(), "AUTH".to_string()).unwrap();
        repo.add(cat1).unwrap();
        repo.add(cat2).unwrap();

        let result = repo.update_abbreviation("AUT", "AUTH");
        assert!(result.is_err());
    }
}
