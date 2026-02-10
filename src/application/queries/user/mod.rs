/// User queries (read operations)
///
/// Queries represent read operations that don't modify state.
/// They are optimized for data retrieval and can be cached.
pub mod get;
pub mod list;
pub mod statistics;

// Re-export query types
pub use get::GetUserQuery;
pub use list::{ListUsersQuery, UserFilters};
pub use statistics::{UserStatistics, UserStatisticsQuery};

// Backward compatibility (deprecated)
#[deprecated(since = "0.3.0", note = "Use `get` module instead")]
pub use get as get_user_query;

#[deprecated(since = "0.3.0", note = "Use `list` module instead")]
pub use list as list_users_query;

#[deprecated(since = "0.3.0", note = "Use `statistics` module instead")]
pub use statistics as user_statistics_query;
