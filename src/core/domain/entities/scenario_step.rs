use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Helper struct for parsing and comparing step orders
/// Supports hierarchical numbering: "1", "2", "3a", "3b", "3a1", "4"
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepOrder {
    /// The numeric base (e.g., 3 for "3", "3a", "3b")
    pub base: u32,
    /// Optional letter suffix (e.g., Some("a") for "3a", Some("b2") for "3b2")
    pub suffix: Option<String>,
}

impl StepOrder {
    /// Parse a step order string into components
    /// Valid formats: "1", "2", "10", "3a", "3b", "3a1", "5xyz99"
    pub fn parse(order: &str) -> Result<Self, String> {
        if order.is_empty() {
            return Err("Step order cannot be empty".to_string());
        }

        // Find where the numeric part ends
        let numeric_end = order
            .chars()
            .position(|c| !c.is_ascii_digit())
            .unwrap_or(order.len());

        if numeric_end == 0 {
            return Err(format!("Step order must start with a number: '{}'", order));
        }

        let base = order[..numeric_end]
            .parse::<u32>()
            .map_err(|_| format!("Invalid numeric base in step order: '{}'", order))?;

        let suffix = if numeric_end < order.len() {
            Some(order[numeric_end..].to_string())
        } else {
            None
        };

        Ok(StepOrder { base, suffix })
    }

    /// Compare two step orders for sorting
    /// Order: numeric base first, then by suffix (None < Some)
    pub fn compare(a: &str, b: &str) -> Ordering {
        let order_a = Self::parse(a).unwrap_or(StepOrder {
            base: u32::MAX,
            suffix: Some(a.to_string()),
        });
        let order_b = Self::parse(b).unwrap_or(StepOrder {
            base: u32::MAX,
            suffix: Some(b.to_string()),
        });

        match order_a.base.cmp(&order_b.base) {
            Ordering::Equal => {
                // Same base number, compare suffixes
                match (&order_a.suffix, &order_b.suffix) {
                    (None, None) => Ordering::Equal,
                    (None, Some(_)) => Ordering::Less, // "3" < "3a"
                    (Some(_), None) => Ordering::Greater, // "3a" > "3"
                    (Some(s1), Some(s2)) => s1.cmp(s2), // "3a" < "3b", "3a1" < "3a2"
                }
            }
            other => other,
        }
    }

    /// Validate that a step order string is in the correct format
    pub fn validate(order: &str) -> Result<(), String> {
        Self::parse(order)?;
        Ok(())
    }

    /// Check if this is a main scenario step (numeric only, no suffix)
    pub fn is_main_step(order: &str) -> bool {
        Self::parse(order)
            .map(|o| o.suffix.is_none())
            .unwrap_or(false)
    }

    /// Check if this is an extension step (has a suffix)
    pub fn is_extension_step(order: &str) -> bool {
        Self::parse(order)
            .map(|o| o.suffix.is_some())
            .unwrap_or(false)
    }
}

/// A single step in a scenario flow
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScenarioStep {
    /// Step order (e.g., "1", "2", "3a", "3b" for hierarchical numbering)
    pub order: String,

    /// Actor call name performing the action (e.g., "guest", "system")
    pub acting_actor: String,

    /// Optional receiving actor call name (who/what receives the action)
    #[serde(default)]
    pub receiving_actor: Option<String>,

    /// What action is performed (e.g., "enters", "verifies", "returns")
    pub action: String,

    /// Full description of what happens (optional - can be auto-generated)
    #[serde(default)]
    pub description: Option<String>,

    /// Additional notes or technical details
    #[serde(default)]
    pub notes: Option<String>,
}

impl ScenarioStep {
    /// Create a new scenario step with acting actor
    pub fn new(order: String, acting_actor: String, action: String) -> Self {
        Self {
            order,
            acting_actor,
            receiving_actor: None,
            action,
            description: None,
            notes: None,
        }
    }

    /// Create a new scenario step with both acting and receiving actors
    pub fn with_receiver(
        order: String,
        acting_actor: String,
        receiving_actor: String,
        action: String,
    ) -> Self {
        Self {
            order,
            acting_actor,
            receiving_actor: Some(receiving_actor),
            action,
            description: None,
            notes: None,
        }
    }

    /// Get the acting actor call name
    pub fn acting_actor(&self) -> &str {
        &self.acting_actor
    }

    /// Get the receiving actor call name if present
    pub fn receiving_actor(&self) -> Option<&str> {
        self.receiving_actor.as_deref()
    }

    /// Set the receiving actor
    pub fn set_receiving_actor(&mut self, receiving_actor: String) {
        self.receiving_actor = Some(receiving_actor);
    }

    /// Clear the receiving actor
    pub fn clear_receiving_actor(&mut self) {
        self.receiving_actor = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_step_creation() {
        let step = ScenarioStep::new(
            "1".to_string(),
            "user".to_string(),
            "enters username and password".to_string(),
        );

        assert_eq!(step.order, "1");
        assert_eq!(step.acting_actor, "user");
        assert_eq!(step.receiving_actor, None);
        assert_eq!(step.action, "enters username and password");
        assert_eq!(step.description, None);
        assert!(step.notes.is_none());
    }

    #[test]
    fn test_scenario_step_with_receiver() {
        let step = ScenarioStep::with_receiver(
            "1".to_string(),
            "user".to_string(),
            "system".to_string(),
            "submits login form".to_string(),
        );

        assert_eq!(step.order, "1");
        assert_eq!(step.acting_actor, "user");
        assert_eq!(step.receiving_actor, Some("system".to_string()));
        assert_eq!(step.action, "submits login form");
        assert_eq!(step.description, None);
    }

    #[test]
    fn test_scenario_step_receiver_methods() {
        let mut step = ScenarioStep::new(
            "1".to_string(),
            "user".to_string(),
            "enters data".to_string(),
        );

        assert_eq!(step.receiving_actor(), None);

        step.set_receiving_actor("database".to_string());
        assert_eq!(step.receiving_actor(), Some("database"));

        step.clear_receiving_actor();
        assert_eq!(step.receiving_actor(), None);
    }

    #[test]
    fn test_scenario_step_equality() {
        let step1 = ScenarioStep::new(
            "1".to_string(),
            "user".to_string(),
            "enters data".to_string(),
        );

        let step2 = ScenarioStep::new(
            "1".to_string(),
            "user".to_string(),
            "enters data".to_string(),
        );

        let step3 = ScenarioStep::new(
            "2".to_string(),
            "user".to_string(),
            "enters data".to_string(),
        );

        assert_eq!(step1, step2);
        assert_ne!(step1, step3);
    }

    #[test]
    fn test_scenario_step_equality_with_receiver() {
        let step1 = ScenarioStep::with_receiver(
            "1".to_string(),
            "user".to_string(),
            "system".to_string(),
            "submits form".to_string(),
        );

        let step2 = ScenarioStep::with_receiver(
            "1".to_string(),
            "user".to_string(),
            "system".to_string(),
            "submits form".to_string(),
        );

        let step3 = ScenarioStep::new(
            "1".to_string(),
            "user".to_string(),
            "submits form".to_string(),
        );

        assert_eq!(step1, step2);
        assert_ne!(step1, step3); // Different because receiver is missing
    }

    #[test]
    fn test_scenario_step_custom_actor() {
        let step = ScenarioStep::new(
            "1".to_string(),
            "payment-gateway".to_string(),
            "processes payment transaction".to_string(),
        );

        assert_eq!(step.acting_actor, "payment-gateway");
    }

    #[test]
    fn test_scenario_step_acting_actor_getter() {
        let step = ScenarioStep::new(
            "1".to_string(),
            "user".to_string(),
            "performs action".to_string(),
        );
        assert_eq!(step.acting_actor(), "user");
    }

    #[test]
    fn test_step_order_parsing() {
        assert!(StepOrder::parse("1").is_ok());
        assert!(StepOrder::parse("10").is_ok());
        assert!(StepOrder::parse("3a").is_ok());
        assert!(StepOrder::parse("3b").is_ok());
        assert!(StepOrder::parse("3a1").is_ok());
        assert!(StepOrder::parse("").is_err());
        assert!(StepOrder::parse("abc").is_err());

        let order = StepOrder::parse("3a").unwrap();
        assert_eq!(order.base, 3);
        assert_eq!(order.suffix, Some("a".to_string()));

        let order = StepOrder::parse("10").unwrap();
        assert_eq!(order.base, 10);
        assert_eq!(order.suffix, None);
    }

    #[test]
    fn test_step_order_comparison() {
        use std::cmp::Ordering;

        // Numeric ordering
        assert_eq!(StepOrder::compare("1", "2"), Ordering::Less);
        assert_eq!(StepOrder::compare("2", "10"), Ordering::Less);
        assert_eq!(StepOrder::compare("10", "2"), Ordering::Greater);

        // Same base, no suffix vs suffix
        assert_eq!(StepOrder::compare("3", "3a"), Ordering::Less);
        assert_eq!(StepOrder::compare("3a", "3"), Ordering::Greater);

        // Same base, different suffixes
        assert_eq!(StepOrder::compare("3a", "3b"), Ordering::Less);
        assert_eq!(StepOrder::compare("3a1", "3a2"), Ordering::Less);
        assert_eq!(StepOrder::compare("3b", "3a"), Ordering::Greater);

        // Complex hierarchical
        assert_eq!(StepOrder::compare("3", "3a"), Ordering::Less);
        assert_eq!(StepOrder::compare("3a", "3a1"), Ordering::Less);
        assert_eq!(StepOrder::compare("3a1", "3b"), Ordering::Less);
        assert_eq!(StepOrder::compare("3b", "4"), Ordering::Less);
    }

    #[test]
    fn test_step_order_is_main_step() {
        assert!(StepOrder::is_main_step("1"));
        assert!(StepOrder::is_main_step("10"));
        assert!(!StepOrder::is_main_step("3a"));
        assert!(!StepOrder::is_main_step("3b1"));
    }

    #[test]
    fn test_step_order_is_extension_step() {
        assert!(!StepOrder::is_extension_step("1"));
        assert!(!StepOrder::is_extension_step("10"));
        assert!(StepOrder::is_extension_step("3a"));
        assert!(StepOrder::is_extension_step("3b1"));
    }
}
