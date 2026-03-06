use super::model::Model;
use super::query::{Mutation, QueryValue, ScyllaQuery};

/// UPDATE operations for any `Model`.
///
/// Mirrors charybdis `Update`. The `update()` method serializes the entire row
/// as the VALUES of the UPDATE statement.
///
/// # Example
/// ```rust
/// user_row.updated_at = UserRow::ts(Utc::now());
/// user_row.update().execute(&session).await?;
/// ```
pub trait Update: Model
where
    Self: 'static,
{
    /// Builds an UPDATE query using this struct's field values.
    ///
    /// The generated query is `UPDATE <table> SET col1=?, col2=?, … WHERE pk=?`.
    /// Field order must exactly match `Model::UPDATE_QUERY`.
    fn update(&self) -> ScyllaQuery<'_, Self, Self, Mutation> {
        ScyllaQuery::new(Self::UPDATE_QUERY, QueryValue::Ref(self))
    }

    /// Convenience — re-fetch after updating using the same primary key.
    ///
    /// Builds an UPDATE and a chained find from the same key values.
    /// This is two operations: call `.update()` first, then `.find_by_primary_key()`.
    fn update_by_primary_key(&self) -> ScyllaQuery<'_, Self::PrimaryKey, Self, Mutation> {
        ScyllaQuery::new(Self::UPDATE_QUERY, QueryValue::Owned(self.primary_key_values()))
    }
}

/// Blanket impl — every `Model` gets `Update` for free.
impl<M: Model + 'static> Update for M {}
