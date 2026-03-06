use scylla::client::caching_session::CachingSession;
use scylla::client::pager::QueryPager;
use scylla::response::query_result::QueryResult;
use scylla::response::{PagingState, PagingStateResponse};
use scylla::serialize::row::SerializeRow;

use super::error::DbError;

/// Execute a query that returns a raw `QueryResult` (mutations, COUNT, etc.).
///
/// Mirrors `charybdis::operations::execute::execute_unpaged`.
pub async fn execute_unpaged(
    session: &CachingSession,
    query: &'static str,
    values: impl SerializeRow,
) -> Result<QueryResult, DbError> {
    session
        .execute_unpaged(query, values)
        .await
        .map_err(|e| DbError::ExecutionError(query, e))
}

/// Execute a query that streams many rows via the driver pager.
///
/// Callers iterate the returned `QueryPager` with `.rows_stream::<Row>()`.
/// Mirrors `charybdis::operations::execute::execute_iter`.
pub async fn execute_iter(
    session: &CachingSession,
    query: &'static str,
    values: impl SerializeRow,
) -> Result<QueryPager, DbError> {
    session
        .execute_iter(query, values)
        .await
        .map_err(|e| DbError::PagerExecutionError(query, Box::new(e)))
}

/// Execute a single page of results with an explicit paging state.
///
/// Returns `(QueryResult, PagingStateResponse)` so the caller can extract the
/// next paging token for cursor-based pagination.
///
/// Mirrors `charybdis::operations::execute::execute_single_page`.
pub async fn execute_single_page(
    session: &CachingSession,
    query: &'static str,
    values: impl SerializeRow,
    paging_state: PagingState,
) -> Result<(QueryResult, PagingStateResponse), DbError> {
    session
        .execute_single_page(query, values, paging_state)
        .await
        .map_err(|e| DbError::ExecutionError(query, e))
}
