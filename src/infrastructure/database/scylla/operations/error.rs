use scylla::errors::{ExecutionError, IntoRowsResultError, RowsError};

/// Database operation errors for the ScyllaDB operations layer.
///
/// Mirrors charybdis's `CharybdisError` but scoped to this project's
/// naming conventions.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Query `{0}` execution failed: {1}")]
    ExecutionError(&'static str, #[source] ExecutionError),

    #[error("Query `{0}` pager execution failed: {1}")]
    PagerExecutionError(&'static str, #[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Query `{0}` into_rows_result failed: {1}")]
    IntoRowsResultError(&'static str, #[source] IntoRowsResultError),

    #[error("Query `{0}` rows() failed: {1}")]
    RowsError(&'static str, #[source] RowsError),

    #[error("Query `{0}` first_row deserialization failed: {1}")]
    DeserializationError(&'static str, #[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("Query `{0}` returned no rows (expected at least one)")]
    NotFoundError(&'static str),
}
