//! Migration command handler for data format changes.

use crate::config::Config;
use crate::core::TomlMigrator;
use anyhow::Result;

/// Handle migration command to update data format.
///
/// Migrates actor files from old ID format (name) to new format (function-name).
///
/// # Arguments
/// * `dry_run` - If true, shows what would be changed without modifying files
///
/// # Returns
/// Ok(()) on success
pub fn handle_migrate_command(dry_run: bool) -> Result<()> {
    let config = Config::load()?;

    println!("🔄 Migrating data to v0.2.0 format...\n");

    if dry_run {
        println!("🔍 DRY RUN MODE - No files will be modified\n");
    }

    match config.storage.backend {
        crate::config::StorageBackend::Toml => {
            // Migrate TOML actor files
            TomlMigrator::migrate_actor_ids_v0_2_0(&config, dry_run)?;
        }
        crate::config::StorageBackend::Sqlite => {
            // SQLite migrations are handled automatically via Migrator
            println!("✅ SQLite migrations are handled automatically on database open.");
            println!("   No manual migration needed.");
        }
    }

    Ok(())
}
