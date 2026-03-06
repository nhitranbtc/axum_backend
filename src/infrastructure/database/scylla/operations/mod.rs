/// Charybdis-style operations layer for the ScyllaDB infrastructure.
///
/// This module provides a typed query builder layer over the raw `scylla` driver,
/// mirroring the architectural patterns of the `charybdis` ORM without depending
/// on that crate.
///
/// # Architecture
///
/// ```text
/// ScyllaQuery<Val, M, Qe>           ← builder returned by operation traits
///     ├── QueryValue<Val>           ← owned / borrowed / empty values enum
///     ├── QueryExecutor<M>          ← execution strategy (RowResult, Stream, …)
///     └── .execute(&CachingSession) ← single async call to run the query
///
/// BaseModel                         ← SELECT constants + key types (every table)
/// Model: BaseModel                  ← INSERT / UPDATE / DELETE constants
///
/// Find: BaseModel    → find_by_primary_key_value(), find_all(), find<Val>(), …
/// Insert: Model      → insert(), insert_if_not_exists()
/// Update: Model      → update()
/// Delete: Model      → delete(), delete_by_partition_key(), delete_by_query<Val>()
/// ```
///
/// # Usage
///
/// ```rust
/// use crate::infrastructure::database::scylla::operations::prelude::*;
///
/// // Find — returns the row or DbError::NotFoundError
/// let user = UserRow::find_by_primary_key_value((user_id,))
///     .execute(&session).await?;
///
/// // Maybe find — returns Option<UserRow>
/// let maybe = UserRow::maybe_find_by_primary_key_value((user_id,))
///     .execute(&session).await?;
///
/// // Insert / update / delete
/// user_row.insert().execute(&session).await?;
/// user_row.update().execute(&session).await?;
/// user_row.delete().execute(&session).await?;
///
/// // Custom query
/// let users: Vec<UserRow> = UserRow::find(UserRow::FIND_BY_EMAIL_QUERY, (email,))
///     .execute(&session).await?;
///
/// // Paged query
/// let (page, paging_state) = UserRow::find_by_partition_key_value_paged(user_id)
///     .page_size(20)
///     .execute(&session).await?;
/// ```
pub mod delete;
pub mod error;
pub mod execute;
pub mod find;
pub mod insert;
pub mod model;
pub mod query;
pub mod update;

/// Convenience re-export — `use operations::prelude::*` brings all traits
/// and types into scope.
pub mod prelude {
    pub use super::delete::Delete;
    pub use super::error::DbError;
    pub use super::execute::{execute_iter, execute_single_page, execute_unpaged};
    pub use super::find::Find;
    pub use super::insert::Insert;
    pub use super::model::{BaseModel, Model};
    pub use super::query::{
        Mutation, OptionalRow, Paged, QueryValue, RowResult, ScyllaQuery, Stream,
    };
    pub use super::update::Update;
}
