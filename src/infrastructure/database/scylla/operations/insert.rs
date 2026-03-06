use super::model::Model;
use super::query::{Mutation, ScyllaQuery, QueryValue};

/// INSERT operations for any `Model`.
///
/// Mirrors charybdis `Insert`. Provides `insert()` and `insert_if_not_exists()`
/// which both return a `ScyllaQuery<'_, Self, Self, Mutation>`.
///
/// # Example
/// ```rust
/// user_row.insert().execute(&session).await?;
/// user_row.insert_if_not_exists().execute(&session).await?;
/// ```
pub trait Insert: Model where Self: 'static {
    /// Builds a full INSERT query using this struct's field values.
    ///
    /// Uses `INSERT INTO … VALUES …` (upsert semantics — overwrites if PK exists).
    fn insert(&self) -> ScyllaQuery<'_, Self, Self, Mutation> {
        ScyllaQuery::new(Self::INSERT_QUERY, QueryValue::Ref(self))
    }

    /// Builds an INSERT … IF NOT EXISTS query.
    ///
    /// Returns `Mutation` (the LWT result is not currently extracted;
    /// use `.execute(session).await` and check no error for success).
    fn insert_if_not_exists(&self) -> ScyllaQuery<'_, Self, Self, Mutation> {
        ScyllaQuery::new(Self::INSERT_IF_NOT_EXISTS_QUERY, QueryValue::Ref(self))
    }
}

/// Blanket impl — every `Model` gets `Insert` for free.
impl<M: Model + 'static> Insert for M {}
