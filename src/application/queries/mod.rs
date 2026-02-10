// Queries (read operations) - CQRS pattern
pub mod user;

pub use user::{GetUserQuery, ListUsersQuery, UserFilters, UserStatistics, UserStatisticsQuery};
