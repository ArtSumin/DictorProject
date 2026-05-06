// application/mod.rs — application layer
// Use cases: orchestration of business logic.
// Analogous to UseCases/ or Interactors/ in an iOS project.

pub mod transcribe;
pub mod queue;
pub mod retry;
pub mod health;
pub mod cache;
pub mod validate;
pub mod export;
