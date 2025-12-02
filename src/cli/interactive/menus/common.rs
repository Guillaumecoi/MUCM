//! # Common Menu Infrastructure
//!
//! Provides reusable components for building type-safe interactive menus.
//!
//! ## Overview
//!
//! Generic menu system using `MenuOption<T>` struct and `display_menu<T>()` function.
//! Ensures type safety by requiring actions to specify their expected context type.

use anyhow::Result;
use inquire::Select;

/// Type alias for menu action closure
pub type MenuAction<T> = Box<dyn Fn(&mut T) -> Result<bool>>;

/// A menu option with display text and associated action
///
/// Pairs human-readable text with an executable closure that receives
/// a mutable reference to a context object of type `T`.
pub struct MenuOption<T> {
    /// The text displayed to the user in the menu
    pub display_text: String,
    /// The action to execute when this option is selected.
    /// Receives a mutable reference to the context and returns whether to exit the menu.
    pub action: MenuAction<T>,
}

impl<T> MenuOption<T> {
    /// Create a new menu option with display text and action
    ///
    /// # Arguments
    ///
    /// * `display_text` - Text shown to user (e.g., "Create use case")
    /// * `action` - Closure executed when selected, receives `&mut T` and returns `Result<bool>`
    ///
    /// # Returns
    ///
    /// A new `MenuOption<T>` instance
    pub fn new<F>(display_text: impl Into<String>, action: F) -> Self
    where
        F: Fn(&mut T) -> Result<bool> + 'static,
    {
        Self {
            display_text: display_text.into(),
            action: Box::new(action),
        }
    }
}

/// Display an interactive menu and execute the selected option
///
/// Presents a list of options to the user, handles selection, and executes
/// the chosen action with the provided context.
///
/// # Arguments
///
/// * `prompt` - Question or instruction displayed above the menu
/// * `options` - Slice of `MenuOption<T>` instances to display
/// * `context` - Mutable reference to context object passed to actions
///
/// # Returns
///
/// * `Ok(true)` if an action requested menu exit
/// * `Ok(false)` if menu should continue (shouldn't happen in normal flow)
/// * `Err(_)` if menu display or action execution failed
///
/// # Type Parameters
///
/// * `T` - The context type that all menu options expect
pub fn display_menu<T>(prompt: &str, options: &[MenuOption<T>], context: &mut T) -> Result<bool> {
    let display_texts: Vec<&str> = options
        .iter()
        .map(|opt| opt.display_text.as_str())
        .collect();

    let choice = Select::new(prompt, display_texts).prompt()?;

    // Find and execute the selected option
    for option in options {
        if option.display_text == choice {
            return (option.action)(context);
        }
    }

    Ok(false) // Should not reach here
}
