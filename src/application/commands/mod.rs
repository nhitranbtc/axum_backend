// Commands (write operations) - CQRS pattern
pub mod user;

pub use user::{CreateUserCommand, UpdateUserCommand};
