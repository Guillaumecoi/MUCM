// Domain entities - Core business objects

// Private modules
mod actor;
mod actor_entity;
mod condition;
mod metadata;
mod methodology_view;
mod persona;
mod reference_type;
mod scenario;
mod scenario_reference;
mod scenario_step;
mod scenario_type;
mod status;
mod use_case;
mod use_case_reference;

// Explicit public exports (visible to parent modules)
pub use actor::Actor;
pub use actor_entity::{ActorEntity, ActorType};
pub use condition::Condition;
pub use metadata::Metadata;
pub use methodology_view::MethodologyView;
pub use persona::Persona;
pub use reference_type::ReferenceType;
pub use scenario::{RepeatBlock, Scenario};
pub use scenario_reference::ScenarioReference;
pub use scenario_step::{ScenarioStep, StepOrder};
pub use scenario_type::ScenarioType;
pub use status::Status;
pub use use_case::{Priority, UseCase};
pub use use_case_reference::UseCaseReference;
