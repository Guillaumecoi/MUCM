//! Status helpers for templates.
//!
//! Provides a small helper to convert status strings/names into emojis.

use handlebars::{Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use std::str::FromStr;

/// Register status-related helpers
pub fn register(handlebars: &mut Handlebars) {
    handlebars.register_helper("status_emoji", Box::new(status_emoji_helper));
}

/// Helper to convert a status name (or pick up from context) to an emoji
/// Usage:
///   {{status_emoji status}}
///   or without param: {{status_emoji}} (reads `aggregated_status` from context)
fn status_emoji_helper(
    h: &Helper,
    _: &Handlebars,
    ctx: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> HelperResult {
    // Try parameter first
    let param_opt = h.param(0).and_then(|v| v.value().as_str()).map(|s| s.to_string());

    let status_str = match param_opt {
        Some(s) if !s.is_empty() => s,
        _ => ctx
            .data()
            .get("aggregated_status")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    };

    if status_str.is_empty() {
        out.write("")?;
        return Ok(());
    }

    // Parse using Status::from_str which accepts variants like "planned", "in_progress", etc.
    match crate::core::domain::Status::from_str(&status_str) {
        Ok(status) => {
            out.write(status.emoji())?;
        }
        Err(_) => {
            // Fallback: try lowercasing and replacing spaces with underscore
            let normal = status_str.to_lowercase().replace(' ', "_");
            match crate::core::domain::Status::from_str(&normal) {
                Ok(status) => out.write(status.emoji())?,
                Err(_) => out.write("")?,
            }
        }
    }

    Ok(())
}
