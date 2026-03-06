use scylla::response::PagingState;
use scylla::serialize::row::SerializeRow;

use super::model::BaseModel;
use super::query::{OptionalRow, Paged, QueryValue, RowResult, ScyllaQuery, Stream};

/// Find operations for any `BaseModel`.
///
/// Mirrors charybdis `Find`. All methods return a `ScyllaQuery` builder that
/// the caller executes via `.execute(session)`.
///
/// # Examples
/// ```rust
/// // Find by primary key — returns M or error if not found
/// let user = UserRow::find_by_primary_key_value((user_id,))
///     .execute(&session).await?;
///
/// // Maybe find — returns Option<M>
/// let maybe = UserRow::maybe_find_by_primary_key_value((user_id,))
///     .execute(&session).await?;
///
/// // Custom query — returns a stream (Vec<M>)
/// let users = UserRow::find(UserRow::FIND_BY_EMAIL_QUERY, (email,))
///     .execute(&session).await?;
/// ```
pub trait Find: BaseModel
where
    Self: 'static,
{
    // ── Custom queries ────────────────────────────────────────────────────────

    /// Run any SELECT query that returns multiple rows.
    fn find<'a, Val: SerializeRow>(
        query: &'static str,
        values: Val,
    ) -> ScyllaQuery<'a, Val, Self, Stream> {
        ScyllaQuery::new(query, QueryValue::Owned(values))
    }

    /// Run any SELECT query with server-side paging.
    fn find_paged<Val: SerializeRow>(
        query: &'static str,
        values: Val,
        paging_state: PagingState,
    ) -> ScyllaQuery<'static, Val, Self, Paged> {
        ScyllaQuery::new(query, QueryValue::Owned(values)).paging_state(paging_state)
    }

    /// Run any SELECT that returns exactly one row.
    fn find_first<'a, Val: SerializeRow>(
        query: &'static str,
        values: Val,
    ) -> ScyllaQuery<'a, Val, Self, RowResult> {
        ScyllaQuery::new(query, QueryValue::Owned(values))
    }

    /// Run any SELECT that returns zero or one rows.
    fn maybe_find_first<'a, Val: SerializeRow>(
        query: &'static str,
        values: Val,
    ) -> ScyllaQuery<'a, Val, Self, OptionalRow> {
        ScyllaQuery::new(query, QueryValue::Owned(values))
    }

    // ── Standard PK / partition key queries ───────────────────────────────────

    /// `SELECT … WHERE <pk_cols> = ?` — must exist or returns `DbError::NotFoundError`.
    fn find_by_primary_key_value(
        value: Self::PrimaryKey,
    ) -> ScyllaQuery<'static, Self::PrimaryKey, Self, RowResult> {
        ScyllaQuery::new(Self::FIND_BY_PRIMARY_KEY_QUERY, QueryValue::Owned(value))
    }

    /// `SELECT … WHERE <pk_cols> = ?` — returns `None` if not found.
    fn maybe_find_by_primary_key_value(
        value: Self::PrimaryKey,
    ) -> ScyllaQuery<'static, Self::PrimaryKey, Self, OptionalRow> {
        ScyllaQuery::new(Self::FIND_BY_PRIMARY_KEY_QUERY, QueryValue::Owned(value))
    }

    /// `SELECT … WHERE <partition_cols> = ?` — returns all matching rows.
    fn find_by_partition_key_value(
        value: Self::PartitionKey,
    ) -> ScyllaQuery<'static, Self::PartitionKey, Self, Stream> {
        ScyllaQuery::new(Self::FIND_BY_PARTITION_KEY_QUERY, QueryValue::Owned(value))
    }

    /// `SELECT … WHERE <partition_cols> = ?` with paging.
    fn find_by_partition_key_value_paged(
        value: Self::PartitionKey,
    ) -> ScyllaQuery<'static, Self::PartitionKey, Self, Paged> {
        ScyllaQuery::new(Self::FIND_BY_PARTITION_KEY_QUERY, QueryValue::Owned(value))
    }

    /// `SELECT … FROM <table>` — returns all rows (use with care on large tables).
    fn find_all<'a>() -> ScyllaQuery<'a, (), Self, Stream> {
        ScyllaQuery::new(Self::FIND_ALL_QUERY, QueryValue::Empty)
    }

    // ── Instance-method variants ──────────────────────────────────────────────

    /// Fetch this row's PK from the database (panics if key columns are unset).
    fn find_by_primary_key(&self) -> ScyllaQuery<'_, Self::PrimaryKey, Self, RowResult> {
        ScyllaQuery::new(
            Self::FIND_BY_PRIMARY_KEY_QUERY,
            QueryValue::Owned(self.primary_key_values()),
        )
    }

    /// Same as above but returns `Option<Self>`.
    fn maybe_find_by_primary_key(&self) -> ScyllaQuery<'_, Self::PrimaryKey, Self, OptionalRow> {
        ScyllaQuery::new(
            Self::FIND_BY_PRIMARY_KEY_QUERY,
            QueryValue::Owned(self.primary_key_values()),
        )
    }

    /// Fetch all rows in this row's partition.
    fn find_by_partition_key(&self) -> ScyllaQuery<'_, Self::PartitionKey, Self, Stream> {
        ScyllaQuery::new(
            Self::FIND_BY_PARTITION_KEY_QUERY,
            QueryValue::Owned(self.partition_key_values()),
        )
    }
}

/// Blanket impl — every `BaseModel + 'static` gets `Find` for free.
impl<M: BaseModel + 'static> Find for M {}
