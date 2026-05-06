// infrastructure/mod.rs — infrastructure layer
// Trait implementations via external deps (reqwest, filesystem).
// Analogous to Services/ or Networking/ in an iOS project.

pub mod stt_client;
pub mod cache;
pub mod audio_validator;
pub mod history;
pub mod exporter;
