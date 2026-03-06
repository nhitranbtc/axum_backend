use scylla::serialize::row::SerializeRow;

use super::model::Model;
use super::query::{Mutation, ScyllaQuery, QueryValue};

/// DELETE operations for any `Model`.
///
/// Mirrors charybdis `Delete`. Provides `delete()` (by full PK),
/// `delete_by_partition_key()` (all rows in a partition), and
/// `delete_by_query<Val>()` for custom DELETE statements.
///
/// # Examples
/// ```rust
/// // Delete by full primary key (derived from this row's fields)
/// user_row.delete().execute(&session).await?;
///
/// // Delete all rows for a partition key
/// user_row.delete_by_partition_key().execute(&session).await?;
///
/// // Custom DELETE with explicit values
/// UserRow::delete_by_query("DELETE FROM users WHERE user_id = ?", (id,))
///     .execute(&session).await?;
/// ```
pub trait Delete: Model where Self: 'static {
    /// Delete this row by its full primary key (partition + clustering).
    fn delete(&self) -> ScyllaQuery<'_, Self::PrimaryKey, Self, Mutation> {
        ScyllaQuery::new(
            Self::DELETE_QUERY,
            QueryValue::Owned(self.primary_key_values()),
        )
    }

    /// Delete **all** rows that share this row's partition key.
    fn delete_by_partition_key(&self) -> ScyllaQuery<'_, Self::PartitionKey, Self, Mutation> {
        ScyllaQuery::new(
            Self::DELETE_BY_PARTITION_KEY_QUERY,
            QueryValue::Owned(self.partition_key_values()),
        )
    }

    /// Run any custom DELETE query with caller-provided values.
    fn delete_by_query<Val: SerializeRow>(
        query: &'static str,
        values: Val,
    ) -> ScyllaQuery<'static, Val, Self, Mutation> {
        ScyllaQuery::new(query, QueryValue::Owned(values))
    }
}

/// Blanket impl — every `Model` gets `Delete` for free.
impl<M: Model + 'static> Delete for M {}
