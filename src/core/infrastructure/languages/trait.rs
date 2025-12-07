//! Language trait definition.
//!
//! Defines the interface that all language implementations must provide.

/// Represents a programming language supported by the system.
///
/// This trait defines the interface that all language implementations must provide.
/// Languages encapsulate metadata about programming languages and their associated
/// test generation templates.
pub trait Language {
    /// Returns the primary name of the language (e.g., "rust", "python").
    fn name(&self) -> &str;

    /// Returns alternative names/aliases for the language (e.g., ["rs"] for Rust).
    /// These can be used for flexible language lookup.
    fn aliases(&self) -> Vec<&str>;

    /// Returns the file extension typically used for this language (e.g., "rs", "py").
    /// Currently unused but reserved for future test file generation features.
    fn file_extension(&self) -> &str;

    /// Returns the Handlebars template content used for generating test files
    /// for this language.
    fn test_template(&self) -> &str;

    /// Returns the comment start syntax for this language (e.g., "#", "//").
    /// Used for parsing safe zones in test files during regeneration.
    fn comment_start(&self) -> &str;
}
