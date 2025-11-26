use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ScenarioType {
    #[default]
    HappyPath, // Main success scenario
    AlternativeFlow, // Valid alternative path
    ExceptionFlow,   // Error/exception handling
    Extension,       // Extension point
}

impl fmt::Display for ScenarioType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ScenarioType::HappyPath => write!(f, "happy_path"),
            ScenarioType::AlternativeFlow => write!(f, "alternative_flow"),
            ScenarioType::ExceptionFlow => write!(f, "exception_flow"),
            ScenarioType::Extension => write!(f, "extension"),
        }
    }
}

impl FromStr for ScenarioType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "happy_path" | "happy" | "main" => Ok(ScenarioType::HappyPath),
            "alternative_flow" | "alternative" | "alt" => Ok(ScenarioType::AlternativeFlow),
            "exception_flow" | "exception" | "error" => Ok(ScenarioType::ExceptionFlow),
            "extension" | "ext" => Ok(ScenarioType::Extension),
            _ => Err(format!("Invalid scenario type: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_type_enum_variants() {
        let happy = ScenarioType::HappyPath;
        assert_eq!(format!("{:?}", happy), "HappyPath");

        let alt = ScenarioType::AlternativeFlow;
        assert_eq!(format!("{:?}", alt), "AlternativeFlow");

        let exc = ScenarioType::ExceptionFlow;
        assert_eq!(format!("{:?}", exc), "ExceptionFlow");

        let ext = ScenarioType::Extension;
        assert_eq!(format!("{:?}", ext), "Extension");
    }

    #[test]
    fn test_scenario_type_display() {
        assert_eq!(ScenarioType::HappyPath.to_string(), "happy_path");
        assert_eq!(
            ScenarioType::AlternativeFlow.to_string(),
            "alternative_flow"
        );
        assert_eq!(ScenarioType::ExceptionFlow.to_string(), "exception_flow");
        assert_eq!(ScenarioType::Extension.to_string(), "extension");
    }

    #[test]
    fn test_scenario_type_from_str_valid() {
        assert_eq!(
            ScenarioType::from_str("happy_path").unwrap(),
            ScenarioType::HappyPath
        );
        assert_eq!(
            ScenarioType::from_str("happy").unwrap(),
            ScenarioType::HappyPath
        );
        assert_eq!(
            ScenarioType::from_str("main").unwrap(),
            ScenarioType::HappyPath
        );

        assert_eq!(
            ScenarioType::from_str("alternative_flow").unwrap(),
            ScenarioType::AlternativeFlow
        );
        assert_eq!(
            ScenarioType::from_str("alternative").unwrap(),
            ScenarioType::AlternativeFlow
        );
        assert_eq!(
            ScenarioType::from_str("alt").unwrap(),
            ScenarioType::AlternativeFlow
        );

        assert_eq!(
            ScenarioType::from_str("exception_flow").unwrap(),
            ScenarioType::ExceptionFlow
        );
        assert_eq!(
            ScenarioType::from_str("exception").unwrap(),
            ScenarioType::ExceptionFlow
        );
        assert_eq!(
            ScenarioType::from_str("error").unwrap(),
            ScenarioType::ExceptionFlow
        );

        assert_eq!(
            ScenarioType::from_str("extension").unwrap(),
            ScenarioType::Extension
        );
        assert_eq!(
            ScenarioType::from_str("ext").unwrap(),
            ScenarioType::Extension
        );
    }

    #[test]
    fn test_scenario_type_from_str_invalid() {
        assert!(ScenarioType::from_str("invalid").is_err());
        assert!(ScenarioType::from_str("unknown").is_err());
        assert!(ScenarioType::from_str("").is_err());

        let error = ScenarioType::from_str("invalid").unwrap_err();
        assert!(error.contains("Invalid scenario type: invalid"));
    }

    #[test]
    fn test_scenario_type_default() {
        let default_type = ScenarioType::default();
        assert_eq!(default_type, ScenarioType::HappyPath);
    }

    #[test]
    fn test_scenario_type_serialization() {
        let original = ScenarioType::AlternativeFlow;
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: ScenarioType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_scenario_type_case_insensitive_parsing() {
        assert_eq!(
            ScenarioType::from_str("HAPPY_PATH").unwrap(),
            ScenarioType::HappyPath
        );
        assert_eq!(
            ScenarioType::from_str("Alternative_Flow").unwrap(),
            ScenarioType::AlternativeFlow
        );
        assert_eq!(
            ScenarioType::from_str("Exception_Flow").unwrap(),
            ScenarioType::ExceptionFlow
        );
        assert_eq!(
            ScenarioType::from_str("EXTENSION").unwrap(),
            ScenarioType::Extension
        );
    }
}
