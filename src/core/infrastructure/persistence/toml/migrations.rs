//! TOML data migration utilities.
//!
//! This module handles migrating TOML-based actor data files when
//! the data format changes (e.g., ID format updates).

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::core::utils::slugify_for_id;

/// TOML data migrator for handling format changes.
pub struct TomlMigrator;

impl TomlMigrator {
    /// Migrate all actor TOML files to new ID format (v0.2.0).
    ///
    /// This updates persona IDs from "name" to "function-name" format,
    /// renaming both files and updating the id field inside each file.
    ///
    /// Example: sarah-chen.toml -> regular-customer-sarah-chen.toml
    ///
    /// # Arguments
    /// * `config` - Project configuration
    /// * `dry_run` - If true, only shows what would be changed without modifying files
    ///
    /// # Returns
    /// Number of files migrated
    pub fn migrate_actor_ids_v0_2_0(config: &Config, dry_run: bool) -> Result<usize> {
        // Check if migration is needed based on config version
        if config.version.as_str() >= "0.2.0" {
            println!(
                "✅ Project is already at version {} - no migration needed.",
                config.version
            );
            return Ok(0);
        }

        // Actor TOML files are in data_dir/actors, not actor_dir (which is for markdown docs)
        let actor_data_dir = Path::new(&config.directories.data_dir).join("actors");

        if !actor_data_dir.exists() {
            return Ok(0);
        }

        let mut migrated_count = 0;

        println!("🔍 Scanning for actors to migrate...");

        // Read all TOML files in actor data directory
        for entry in fs::read_dir(&actor_data_dir).context("Failed to read actor data directory")? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) != Some("toml") {
                continue;
            }

            // Read the TOML file
            let content =
                fs::read_to_string(&path).with_context(|| format!("Failed to read {:?}", path))?;

            let mut data: toml::Value =
                toml::from_str(&content).with_context(|| format!("Failed to parse {:?}", path))?;

            // Check if it's a persona
            let actor_type = data
                .get("actor_type")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if !actor_type.eq_ignore_ascii_case("persona") {
                // Skip system actors
                continue;
            }

            // Get current ID, name, and function
            let old_id = data
                .get("id")
                .and_then(|v| v.as_str())
                .context("Actor missing id field")?
                .to_string();

            let name = data
                .get("name")
                .and_then(|v| v.as_str())
                .context("Actor missing name field")?
                .to_string();

            // Check for function at root level or in extra section
            let function = data
                .get("function")
                .and_then(|v| v.as_str())
                .or_else(|| {
                    data.get("extra")
                        .and_then(|extra| extra.get("function"))
                        .and_then(|v| v.as_str())
                })
                .unwrap_or("");

            if function.is_empty() {
                println!("   ⚠️  Skipping {:?} - no function field", path.file_name());
                continue;
            }

            // Generate new ID: function-name
            let new_id = format!("{}-{}", slugify_for_id(function), slugify_for_id(&name));

            // Check if already migrated
            if old_id == new_id {
                continue;
            }

            migrated_count += 1;

            let old_filename = path.file_name().unwrap().to_str().unwrap();
            let new_filename = format!("{}.toml", new_id);
            let new_path = actor_data_dir.join(&new_filename);

            println!("   {} -> {}", old_filename, new_filename);

            if !dry_run {
                // Update the id field in the TOML
                if let Some(table) = data.as_table_mut() {
                    table.insert("id".to_string(), toml::Value::String(new_id.clone()));
                }

                // Write updated content
                let new_content =
                    toml::to_string_pretty(&data).context("Failed to serialize TOML")?;

                fs::write(&new_path, new_content)
                    .with_context(|| format!("Failed to write {:?}", new_path))?;

                // Remove old file
                fs::remove_file(&path).with_context(|| format!("Failed to remove {:?}", path))?;

                // Also migrate markdown file if it exists
                let docs_dir = Path::new(&config.directories.actor_dir)
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| Path::new("docs").to_path_buf());

                let old_md_path = docs_dir.join(format!("{}.md", old_id));
                let new_md_path = docs_dir.join(format!("{}.md", new_id));

                if old_md_path.exists() {
                    fs::rename(&old_md_path, &new_md_path)
                        .with_context(|| format!("Failed to rename markdown {:?}", old_md_path))?;
                }
            }
        }

        if dry_run {
            println!(
                "\n✅ Dry run complete. {} actor(s) would be migrated.",
                migrated_count
            );
            println!("   Run without --dry-run to apply changes.");
        } else if migrated_count > 0 {
            println!(
                "\n✅ Migrated {} actor(s) to new ID format.",
                migrated_count
            );

            // Update config version after successful migration
            let config_path = crate::config::Config::config_path();
            if config_path.exists() {
                let content = fs::read_to_string(&config_path)?;
                let updated_content = if content.contains("version =") {
                    // Replace existing version
                    content
                        .lines()
                        .map(|line| {
                            if line.trim_start().starts_with("version =") {
                                format!("version = \"{}\"", crate::config::Config::CONFIG_VERSION)
                            } else {
                                line.to_string()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    // Add version at the top
                    format!(
                        "version = \"{}\"\n\n{}",
                        crate::config::Config::CONFIG_VERSION,
                        content
                    )
                };
                fs::write(&config_path, updated_content)?;
                println!(
                    "   Updated config version to {}",
                    crate::config::Config::CONFIG_VERSION
                );
            }
        } else {
            println!("\n✅ No actors need migration.");
        }

        Ok(migrated_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config(temp_dir: &TempDir) -> Result<Config> {
        let data_dir = temp_dir.path().join("use-cases-data");
        let actor_dir = data_dir.join("actors");
        fs::create_dir_all(&actor_dir)?;

        let config_content = format!(
            r#"version = "0.1.0"

[project]
name = "Test Project"
description = "Test project"

[directories]
use_case_dir = "docs/use-cases"
test_dir = "tests"
actor_dir = "docs/actors"
data_dir = "{}"

[templates]
methodologies = ["developer"]
default_methodology = "developer"
default_scenario_template = "scenarios/scenario.hbs"

[generation]
test_language = "none"
auto_generate_tests = false
overwrite_test_documentation = false

[storage]
backend = "toml"

[metadata]
created = true
last_updated = true
"#,
            data_dir.display()
        );

        fs::create_dir_all(temp_dir.path().join(".config/.mucm"))?;
        fs::write(
            temp_dir.path().join(".config/.mucm/mucm.toml"),
            config_content,
        )?;

        std::env::set_current_dir(temp_dir.path())?;
        Config::load()
    }

    fn create_test_actor(
        dir: &Path,
        id: &str,
        name: &str,
        function: &str,
        actor_type: &str,
    ) -> Result<()> {
        let content = format!(
            r#"
id = "{}"
name = "{}"
actor_type = "{}"
emoji = "🙂"
function = "{}"

[metadata]
created_at = "2025-12-01T00:00:00Z"
updated_at = "2025-12-01T00:00:00Z"
"#,
            id, name, actor_type, function
        );

        fs::write(dir.join(format!("{}.toml", id)), content)?;
        Ok(())
    }

    #[test]
    fn test_migrate_persona_id() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = create_test_config(&temp_dir)?;
        let actor_data_dir = Path::new(&config.directories.data_dir).join("actors");

        // Create old-format persona
        create_test_actor(
            &actor_data_dir,
            "sarah-chen",
            "Sarah Chen",
            "Regular Customer",
            "persona",
        )?;

        // Run migration
        let count = TomlMigrator::migrate_actor_ids_v0_2_0(&config, false)?;
        assert_eq!(count, 1);

        // Check new file exists
        let new_file = actor_data_dir.join("regular-customer-sarah-chen.toml");
        assert!(new_file.exists());

        // Check old file is gone
        let old_file = actor_data_dir.join("sarah-chen.toml");
        assert!(!old_file.exists());

        // Check ID in file is updated
        let content = fs::read_to_string(&new_file)?;
        assert!(content.contains("id = \"regular-customer-sarah-chen\""));

        Ok(())
    }

    #[test]
    fn test_skip_system_actors() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = create_test_config(&temp_dir)?;
        let actor_data_dir = Path::new(&config.directories.data_dir).join("actors");

        // Create system actor
        create_test_actor(
            &actor_data_dir,
            "database",
            "Database",
            "Storage",
            "database",
        )?;

        // Run migration
        let count = TomlMigrator::migrate_actor_ids_v0_2_0(&config, false)?;
        assert_eq!(count, 0);

        // Check system actor file unchanged
        assert!(actor_data_dir.join("database.toml").exists());

        Ok(())
    }

    #[test]
    fn test_dry_run() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config = create_test_config(&temp_dir)?;
        let actor_data_dir = Path::new(&config.directories.data_dir).join("actors");

        // Create old-format persona
        create_test_actor(
            &actor_data_dir,
            "john-doe",
            "John Doe",
            "Admin User",
            "persona",
        )?;

        // Run dry run
        let count = TomlMigrator::migrate_actor_ids_v0_2_0(&config, true)?;
        assert_eq!(count, 1);

        // Old file should still exist
        assert!(actor_data_dir.join("john-doe.toml").exists());

        // New file should NOT exist
        assert!(!actor_data_dir.join("admin-user-john-doe.toml").exists());

        Ok(())
    }
}
