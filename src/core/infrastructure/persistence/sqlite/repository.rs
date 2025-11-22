//! SQLite implementation of the UseCaseRepository trait.
//!
//! This module provides a complete SQLite-based repository for use cases,
//! implementing all methods from the UseCaseRepository trait with proper
//! transaction support and error handling.

use crate::core::domain::{Condition, UseCase};
use crate::core::infrastructure::persistence::sqlite::Migrator;
use crate::core::infrastructure::persistence::traits::UseCaseRepository;
use anyhow::{anyhow, Context, Result};
use rusqlite::{params, Connection, Transaction};
use std::path::Path;
use std::sync::Mutex;

/// SQLite-based repository for use cases.
///
/// Provides full CRUD operations, querying, batch operations, and transaction
/// support for use case persistence.
///
/// # Thread Safety
/// Uses `Mutex<Connection>` for thread-safe database access.
///
/// # Error Handling
/// All methods return `anyhow::Result` with descriptive error messages.
///
/// # Performance
/// - Indexed queries for fast lookups
/// - Batch operations for efficiency
/// - Connection pooling via Mutex
pub struct SqliteUseCaseRepository {
    /// Thread-safe database connection
    conn: Mutex<Connection>,
    /// Path to the database file (used for relative markdown storage)
    db_path: std::path::PathBuf,
}

impl SqliteUseCaseRepository {
    /// Create a new SQLite repository with database at the given path.
    ///
    /// # Arguments
    /// * `db_path` - Path to the SQLite database file
    ///
    /// # Returns
    /// New repository instance, or error if database setup fails
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let db_path_buf = db_path.as_ref().to_path_buf();
        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open database at {:?}", db_path.as_ref()))?;

        // Enable foreign keys for data integrity
        conn.execute("PRAGMA foreign_keys = ON", [])
            .context("Failed to enable foreign keys")?;

        // Run migrations to ensure schema is up to date
        Migrator::migrate(&conn).context("Failed to run database migrations")?;

        Ok(Self {
            conn: Mutex::new(conn),
            db_path: db_path_buf,
        })
    }

    /// Get a connection from the mutex (internal helper).
    fn get_conn(&self) -> Result<std::sync::MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|e| anyhow!("Failed to acquire database lock: {}", e))
    }

    /// Execute a query within a database transaction (internal helper).
    fn with_db_transaction<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Transaction) -> Result<T>,
    {
        let mut conn = self.get_conn()?;
        let tx = conn.transaction().context("Failed to start transaction")?;
        let result = f(&tx)?;
        tx.commit().context("Failed to commit transaction")?;
        Ok(result)
    }

    /// Load scenarios for a use case from relational tables (Connection version).
    fn load_scenarios_for_use_case(
        conn: &Connection,
        use_case_id: &str,
    ) -> Result<Vec<crate::core::domain::Scenario>> {
        use crate::core::domain::{Scenario, ScenarioReference, ScenarioStep};

        let mut scenarios = Vec::new();

        // Load all scenarios for this use case
        let mut stmt = conn.prepare(
            "SELECT id, title, description, scenario_type, status, persona, created_at, updated_at, extra_json
             FROM scenarios WHERE use_case_id = ? ORDER BY id"
        )?;

        let scenario_rows = stmt.query_map([use_case_id], |row| {
            let scenario_id: String = row.get(0)?;
            let extra_json: String = row.get(8)?;
            let extra: std::collections::HashMap<String, serde_json::Value> =
                serde_json::from_str(&extra_json).unwrap_or_default();

            Ok((
                scenario_id,
                (
                    row.get::<_, String>(1)?,         // title
                    row.get::<_, String>(2)?,         // description
                    row.get::<_, String>(3)?,         // scenario_type
                    row.get::<_, String>(4)?,         // status
                    row.get::<_, Option<String>>(5)?, // persona
                    row.get::<_, String>(6)?,         // created_at
                    row.get::<_, String>(7)?,         // updated_at
                    extra,
                ),
            ))
        })?;

        for scenario_result in scenario_rows {
            let (
                scenario_id,
                (
                    title,
                    description,
                    scenario_type_str,
                    status_str,
                    persona,
                    _created_at_str,
                    _updated_at_str,
                    extra,
                ),
            ) = scenario_result?;

            // Parse scenario type and status
            let scenario_type = scenario_type_str.parse().unwrap_or_default();
            let status = crate::core::domain::Status::from_str(&status_str)
                .unwrap_or(crate::core::domain::Status::Planned);

            // Load steps
            let mut steps_stmt = conn.prepare(
                "SELECT step_order, actor, receiver, action, description, notes FROM scenario_steps WHERE scenario_id = ? ORDER BY step_order"
            )?;
            let step_rows = steps_stmt.query_map([&scenario_id], |row| {
                Ok(ScenarioStep {
                    order: row.get(0)?,
                    actor: row.get(1)?,
                    receiver: row.get(2)?,
                    action: row.get(3)?,
                    description: row.get(4)?,
                    notes: row.get(5)?,
                })
            })?;
            let steps: Vec<ScenarioStep> = step_rows.collect::<Result<Vec<_>, _>>()?;

            // Load preconditions
            let mut precond_stmt = conn.prepare(
                "SELECT condition_text, target_type, target_id, relationship FROM scenario_preconditions WHERE scenario_id = ? ORDER BY condition_order"
            )?;
            let precond_rows = precond_stmt.query_map([&scenario_id], |row| {
                let text: String = row.get(0)?;
                let target_type_str: Option<String> = row.get(1)?;
                let target_id: Option<String> = row.get(2)?;
                let relationship: Option<String> = row.get(3)?;

                let target_type = target_type_str.and_then(|s| s.parse().ok());

                Ok(Condition {
                    text,
                    target_type,
                    target_id,
                    relationship,
                })
            })?;
            let preconditions: Vec<Condition> = precond_rows.collect::<Result<Vec<_>, _>>()?;

            // Load postconditions
            let mut postcond_stmt = conn.prepare(
                "SELECT condition_text, target_type, target_id, relationship FROM scenario_postconditions WHERE scenario_id = ? ORDER BY condition_order"
            )?;
            let postcond_rows = postcond_stmt.query_map([&scenario_id], |row| {
                let text: String = row.get(0)?;
                let target_type_str: Option<String> = row.get(1)?;
                let target_id: Option<String> = row.get(2)?;
                let relationship: Option<String> = row.get(3)?;

                let target_type = target_type_str.and_then(|s| s.parse().ok());

                Ok(Condition {
                    text,
                    target_type,
                    target_id,
                    relationship,
                })
            })?;
            let postconditions: Vec<Condition> = postcond_rows.collect::<Result<Vec<_>, _>>()?;

            // Load references
            let mut ref_stmt = conn.prepare(
                "SELECT ref_type, target_id, relationship, description FROM scenario_references WHERE scenario_id = ? ORDER BY id"
            )?;
            let ref_rows = ref_stmt.query_map([&scenario_id], |row| {
                let ref_type_str: String = row.get(0)?;
                let ref_type = ref_type_str
                    .parse()
                    .unwrap_or(crate::core::domain::ReferenceType::UseCase);
                Ok(ScenarioReference {
                    ref_type,
                    target_id: row.get(1)?,
                    relationship: row.get(2)?,
                    description: row.get(3)?,
                })
            })?;
            let references: Vec<ScenarioReference> = ref_rows.collect::<Result<Vec<_>, _>>()?;

            // For backward compatibility, derive primary_actor from first step or default to User
            let primary_actor = steps
                .first()
                .map(|s| s.actor.clone())
                .unwrap_or(crate::core::domain::Actor::User);

            scenarios.push(Scenario {
                id: scenario_id,
                title,
                description,
                scenario_type,
                status,
                is_main: true, // Default to main for existing scenarios
                primary_actor,
                persona,
                extends_scenario_id: None,
                extends_at_step: None,
                returns_at_step: None,
                steps,
                repeat_blocks: Vec::new(), // Empty for existing scenarios
                preconditions,
                postconditions,
                references,
                extra,
            });
        }

        Ok(scenarios)
    }

    /// Save a use case to the database (internal implementation).
    fn save_internal(tx: &Transaction, use_case: &UseCase) -> Result<()> {
        // Serialize extra fields to JSON (scenarios are now in separate tables)
        let extra_json = serde_json::to_string(&use_case.extra)
            .context("Failed to serialize extra fields to JSON")?;

        // Insert or replace the main use case record
        tx.execute(
            r#"
            INSERT OR REPLACE INTO use_cases (
                id, title, category, description, priority,
                created_at, updated_at, extra_json
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params![
                use_case.id,
                use_case.title,
                use_case.category,
                use_case.description,
                use_case.priority.to_string().to_uppercase(),
                use_case.metadata.created_at.to_rfc3339(),
                use_case.metadata.updated_at.to_rfc3339(),
                extra_json,
            ],
        )
        .context("Failed to save use case")?;

        // Clear existing preconditions, postconditions, and references
        tx.execute(
            "DELETE FROM use_case_preconditions WHERE use_case_id = ?",
            [&use_case.id],
        )
        .context("Failed to clear existing preconditions")?;
        tx.execute(
            "DELETE FROM use_case_postconditions WHERE use_case_id = ?",
            [&use_case.id],
        )
        .context("Failed to clear existing postconditions")?;
        tx.execute(
            "DELETE FROM use_case_references WHERE use_case_id = ?",
            [&use_case.id],
        )
        .context("Failed to clear existing references")?;

        // Clear existing scenarios (CASCADE will delete related data)
        tx.execute(
            "DELETE FROM scenarios WHERE use_case_id = ?",
            [&use_case.id],
        )
        .context("Failed to clear existing scenarios")?;

        // Insert preconditions
        for (index, precondition) in use_case.preconditions.iter().enumerate() {
            tx.execute(
                "INSERT INTO use_case_preconditions (use_case_id, condition_order, condition_text, target_type, target_id, relationship) VALUES (?, ?, ?, ?, ?, ?)",
                params![
                    use_case.id,
                    index as i32,
                    &precondition.text,
                    precondition.target_type.as_ref().map(|t| t.to_string()),
                    &precondition.target_id,
                    &precondition.relationship
                ],
            )
            .context("Failed to save precondition")?;
        }

        // Insert postconditions
        for (index, postcondition) in use_case.postconditions.iter().enumerate() {
            tx.execute(
                "INSERT INTO use_case_postconditions (use_case_id, condition_order, condition_text, target_type, target_id, relationship) VALUES (?, ?, ?, ?, ?, ?)",
                params![
                    use_case.id,
                    index as i32,
                    &postcondition.text,
                    postcondition.target_type.as_ref().map(|t| t.to_string()),
                    &postcondition.target_id,
                    &postcondition.relationship
                ],
            )
            .context("Failed to save postcondition")?;
        }

        // Insert use case references
        for reference in &use_case.use_case_references {
            tx.execute(
                "INSERT INTO use_case_references (use_case_id, target_id, relationship, description) VALUES (?, ?, ?, ?)",
                params![use_case.id, reference.target_id, reference.relationship, reference.description],
            )
            .context("Failed to save use case reference")?;
        }

        // Insert scenarios
        for scenario in &use_case.scenarios {
            // Serialize scenario extra fields
            let scenario_extra_json = serde_json::to_string(&scenario.extra)
                .context("Failed to serialize scenario extra fields")?;

            tx.execute(
                "INSERT INTO scenarios (id, use_case_id, title, description, scenario_type, status, persona, extra_json)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    scenario.id,
                    use_case.id,
                    scenario.title,
                    scenario.description,
                    scenario.scenario_type.to_string(),
                    scenario.status.to_string(),
                    scenario.persona,
                    scenario_extra_json,
                ],
            )
            .context("Failed to save scenario")?;

            // Insert scenario steps
            for step in &scenario.steps {
                tx.execute(
                    "INSERT INTO scenario_steps (scenario_id, step_order, actor, receiver, action, description, notes)
                     VALUES (?, ?, ?, ?, ?, ?, ?)",
                    params![scenario.id, step.order, step.actor, step.receiver, step.action, step.description, step.notes],
                )
                .context("Failed to save scenario step")?;
            }

            // Insert scenario preconditions
            for (index, precondition) in scenario.preconditions.iter().enumerate() {
                tx.execute(
                    "INSERT INTO scenario_preconditions (scenario_id, condition_order, condition_text, target_type, target_id, relationship)
                     VALUES (?, ?, ?, ?, ?, ?)",
                    params![
                        scenario.id,
                        index as i32,
                        &precondition.text,
                        precondition.target_type.as_ref().map(|t| t.to_string()),
                        &precondition.target_id,
                        &precondition.relationship
                    ],
                )
                .context("Failed to save scenario precondition")?;
            }

            // Insert scenario postconditions
            for (index, postcondition) in scenario.postconditions.iter().enumerate() {
                tx.execute(
                    "INSERT INTO scenario_postconditions (scenario_id, condition_order, condition_text, target_type, target_id, relationship)
                     VALUES (?, ?, ?, ?, ?, ?)",
                    params![
                        scenario.id,
                        index as i32,
                        &postcondition.text,
                        postcondition.target_type.as_ref().map(|t| t.to_string()),
                        &postcondition.target_id,
                        &postcondition.relationship
                    ],
                )
                .context("Failed to save scenario postcondition")?;
            }

            // Insert scenario references
            for reference in &scenario.references {
                tx.execute(
                    "INSERT INTO scenario_references (scenario_id, ref_type, target_id, relationship, description)
                     VALUES (?, ?, ?, ?, ?)",
                    params![
                        scenario.id,
                        reference.ref_type.to_string(),
                        reference.target_id,
                        reference.relationship,
                        reference.description,
                    ],
                )
                .context("Failed to save scenario reference")?;
            }
        }

        Ok(())
    }

    /// Load a use case from the database (internal implementation for Connection).
    fn load_by_id_internal_conn(conn: &Connection, id: &str) -> Result<Option<UseCase>> {
        // Query the main use case record
        let mut stmt = conn
            .prepare(
                r#"
            SELECT id, title, category, description, priority,
                   created_at, updated_at, extra_json
            FROM use_cases WHERE id = ?
            "#,
            )
            .context("Failed to prepare use case query")?;

        let mut rows = stmt
            .query_map([id], |row| {
                let extra_json: String = row.get(7)?;
                let extra: std::collections::HashMap<String, serde_json::Value> =
                    serde_json::from_str(&extra_json).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            7,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?;

                Ok(UseCase {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    category: row.get(2)?,
                    description: row.get(3)?,
                    priority: row.get::<_, String>(4)?.parse().map_err(|e: String| {
                        rusqlite::Error::FromSqlConversionFailure(
                            4,
                            rusqlite::types::Type::Text,
                            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
                        )
                    })?,
                    metadata: crate::core::domain::Metadata {
                        created_at: row.get::<_, String>(5)?.parse().map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                5,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?,
                        updated_at: row.get::<_, String>(6)?.parse().map_err(|e| {
                            rusqlite::Error::FromSqlConversionFailure(
                                6,
                                rusqlite::types::Type::Text,
                                Box::new(e),
                            )
                        })?,
                    },
                    views: Vec::new(), // Will be populated below (multi-view support)
                    preconditions: Vec::new(), // Will be populated below
                    postconditions: Vec::new(), // Will be populated below
                    methodology_fields: std::collections::HashMap::new(), // New field for methodology-specific fields
                    use_case_references: Vec::new(),                      // Will be populated below
                    scenarios: Vec::new(), // Will be loaded from relational tables
                    extra,
                })
            })
            .context("Failed to execute use case query")?;

        let mut use_case = match rows.next() {
            Some(row) => row.context("Failed to read use case row")?,
            None => return Ok(None),
        };

        // Load preconditions
        let mut preconditions = Vec::new();
        let mut stmt = conn.prepare(
            "SELECT condition_text, target_type, target_id, relationship FROM use_case_preconditions WHERE use_case_id = ? ORDER BY condition_order"
        )
        .context("Failed to prepare preconditions query")?;
        let precondition_rows = stmt
            .query_map([id], |row| {
                let text: String = row.get(0)?;
                let target_type_str: Option<String> = row.get(1)?;
                let target_id: Option<String> = row.get(2)?;
                let relationship: Option<String> = row.get(3)?;

                let target_type = target_type_str.and_then(|s| s.parse().ok());

                Ok(Condition {
                    text,
                    target_type,
                    target_id,
                    relationship,
                })
            })
            .context("Failed to execute preconditions query")?;
        for condition in precondition_rows {
            preconditions.push(condition.context("Failed to read precondition")?);
        }
        use_case.preconditions = preconditions;

        // Load postconditions
        let mut postconditions = Vec::new();
        let mut stmt = conn.prepare(
            "SELECT condition_text, target_type, target_id, relationship FROM use_case_postconditions WHERE use_case_id = ? ORDER BY condition_order"
        )
        .context("Failed to prepare postconditions query")?;
        let postcondition_rows = stmt
            .query_map([id], |row| {
                let text: String = row.get(0)?;
                let target_type_str: Option<String> = row.get(1)?;
                let target_id: Option<String> = row.get(2)?;
                let relationship: Option<String> = row.get(3)?;

                let target_type = target_type_str.and_then(|s| s.parse().ok());

                Ok(Condition {
                    text,
                    target_type,
                    target_id,
                    relationship,
                })
            })
            .context("Failed to execute postconditions query")?;
        for condition in postcondition_rows {
            postconditions.push(condition.context("Failed to read postcondition")?);
        }
        use_case.postconditions = postconditions;

        // Load references
        let mut references = Vec::new();
        let mut stmt = conn
            .prepare(
                "SELECT target_id, relationship, description FROM use_case_references WHERE use_case_id = ? ORDER BY id",
            )
            .context("Failed to prepare references query")?;
        let reference_rows = stmt
            .query_map([id], |row| {
                Ok(crate::core::domain::UseCaseReference {
                    target_id: row.get(0)?,
                    relationship: row.get(1)?,
                    description: row.get(2)?,
                })
            })
            .context("Failed to execute references query")?;
        for reference in reference_rows {
            references.push(reference.context("Failed to read reference")?);
        }
        use_case.use_case_references = references;

        // Load scenarios from relational tables
        let scenarios = Self::load_scenarios_for_use_case(conn, id)?;
        use_case.scenarios = scenarios;

        Ok(Some(use_case))
    }
}

impl UseCaseRepository for SqliteUseCaseRepository {
    fn save(&self, use_case: &UseCase) -> Result<()> {
        self.with_db_transaction(|tx| Self::save_internal(tx, use_case))
    }

    fn load_all(&self) -> Result<Vec<UseCase>> {
        let conn = self.get_conn()?;
        let mut stmt = conn
            .prepare("SELECT id FROM use_cases ORDER BY id")
            .context("Failed to prepare load_all query")?;

        let ids = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .context("Failed to execute load_all query")?
            .collect::<Result<Vec<String>, _>>()
            .context("Failed to collect use case IDs")?;

        let mut use_cases = Vec::new();
        for id in ids {
            if let Some(mut use_case) = Self::load_by_id_internal_conn(&conn, &id)
                .with_context(|| format!("Failed to load use case {}", id))?
            {
                // Migration: If use case has no views, add default view
                if use_case.views.is_empty() {
                    use crate::core::MethodologyView;
                    let config = crate::config::Config::load()
                        .unwrap_or_else(|_| crate::config::Config::default());
                    let default_methodology = &config.templates.default_methodology;
                    let default_view = MethodologyView::new(default_methodology, "normal");
                    use_case.views.push(default_view);

                    // Note: Auto-save happens via the application service layer
                    // which will detect the change and persist it
                }

                use_cases.push(use_case);
            }
        }

        Ok(use_cases)
    }

    fn load_by_id(&self, id: &str) -> Result<Option<UseCase>> {
        let conn = self.get_conn()?;
        Self::load_by_id_internal_conn(&conn, id)
    }

    fn save_markdown(&self, use_case_id: &str, content: &str) -> Result<()> {
        // Save markdown files in a directory relative to the database location
        // This ensures test isolation and proper organization
        let db_dir = self.db_path.parent().unwrap_or(std::path::Path::new("."));
        let markdown_dir = db_dir.join("markdown");
        std::fs::create_dir_all(&markdown_dir)
            .with_context(|| format!("Failed to create markdown directory {:?}", markdown_dir))?;

        let filename = format!("{}.md", use_case_id);
        let filepath = markdown_dir.join(filename);
        std::fs::write(&filepath, content)
            .with_context(|| format!("Failed to write markdown file {:?}", filepath))?;

        Ok(())
    }

    fn save_markdown_with_filename(
        &self,
        _use_case: &UseCase,
        filename: &str,
        content: &str,
    ) -> Result<()> {
        let db_dir = self.db_path.parent().unwrap_or(std::path::Path::new("."));
        let markdown_dir = db_dir.join("markdown");
        std::fs::create_dir_all(&markdown_dir)
            .with_context(|| format!("Failed to create markdown directory {:?}", markdown_dir))?;

        let filepath = markdown_dir.join(filename);
        std::fs::write(&filepath, content)
            .with_context(|| format!("Failed to write markdown file {:?}", filepath))?;

        Ok(())
    }
}
