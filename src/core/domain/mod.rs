// Domain layer - Pure business logic, framework agnostic

mod entities;
mod repositories;
mod services;

// Re-exports
pub use entities::{
    Actor, ActorEntity, ActorType, Category, Condition, Metadata, MethodologyView, Persona,
    Priority, ReferenceType, RepeatBlock, Scenario, ScenarioReference, ScenarioStep,
    ScenarioType, Status, StepOrder, UseCase, UseCaseReference,
};
pub use repositories::{ActorRepository, PersonaRepository};
pub use services::{
    ExtensionPointUpdater, ScenarioFlowValidator, ScenarioReferenceValidator, UseCaseService,
};
