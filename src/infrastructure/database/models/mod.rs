/// Database models organized by domain
///
/// This module contains all database models (Diesel structs) organized by domain.
/// Models are separate from domain entities to maintain clean architecture.
pub mod auth;
pub mod common;
pub mod user;

// Re-export models for convenience
pub use auth::RefreshTokenModel;
pub use user::UserModel;

// Re-export common traits
pub use common::{HasUuid, SoftDeletable, Timestamped};
