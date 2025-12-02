// Application layer - orchestrates use cases and business logic
pub mod creators;
pub mod generators;
pub mod methodology_field_collector;
pub mod services;
mod use_case_coordinator;

pub use methodology_field_collector::MethodologyFieldCollector;
pub use use_case_coordinator::{
    AddScenarioParams, AddScenarioStepParams, CreateExtensionScenarioParams,
    CreateUseCaseWithViewsParams, InsertStepWithExtensionParams, UseCaseCoordinator,
};

#[cfg(test)]
pub mod testing;
