// lib.rs — public API of the dictor-cli crate.
// Declares all Clean Architecture layers.

pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod interface;
pub mod ffi;

uniffi::setup_scaffolding!();
