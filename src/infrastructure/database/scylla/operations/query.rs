#![allow(clippy::manual_async_fn)]
use scylla::client::caching_session::CachingSession;
use scylla::errors::FirstRowError;
use scylla::response::PagingState;
use scylla::response::PagingStateResponse;
use scylla::serialize::row::SerializeRow;
use scylla::statement::Statement;

use super::error::DbError;
use super::model::BaseModel;

// ── Marker types ──────────────────────────────────────────────────────────────

/// Marker: query returns exactly one typed row (error if empty).
pub struct RowResult;
/// Marker: query returns zero or one typed row.
pub struct OptionalRow;
/// Marker: query streams many typed rows.
pub struct Stream;
/// Marker: query returns a page of typed rows + continuation.
pub struct Paged;
/// Marker: query is a write/mutation (INSERT, UPDATE, DELETE, TRUNCATE).
pub struct Mutation;

// ── QueryType ─────────────────────────────────────────────────────────────────

/// Associates a marker type with the Rust type the query ultimately produces.
pub trait QueryType<M: BaseModel> {
    type Output;
}

impl<M: BaseModel> QueryType<M> for RowResult {
    type Output = M;
}
impl<M: BaseModel> QueryType<M> for OptionalRow {
    type Output = Option<M>;
}
impl<M: BaseModel + 'static> QueryType<M> for Stream {
    type Output = Vec<M>;
}
impl<M: BaseModel> QueryType<M> for Paged {
    type Output = (Vec<M>, PagingStateResponse);
}
impl<M: BaseModel> QueryType<M> for Mutation {
    type Output = ();
}

// ── QueryExecutor ─────────────────────────────────────────────────────────────

/// Async execution strategy for a `ScyllaQuery`. Each marker type provides its
/// own `execute` implementation — callers see only `.execute(session)`.
pub trait QueryExecutor<M: BaseModel>: QueryType<M> {
    fn execute<'a, Val, Qe>(
        query: ScyllaQuery<'a, Val, M, Qe>,
        session: &'a CachingSession,
    ) -> impl std::future::Future<Output = Result<Self::Output, DbError>> + Send + 'a
    where
        M: BaseModel + Send + Sync + 'static,
        Val: SerializeRow + Send + Sync,
        Qe: QueryExecutor<M> + Send,
        Self: Sized;
}

impl<M: BaseModel + Send + Sync + 'static> QueryExecutor<M> for RowResult {
    fn execute<'a, Val, Qe>(
        query: ScyllaQuery<'a, Val, M, Qe>,
        session: &'a CachingSession,
    ) -> impl std::future::Future<Output = Result<M, DbError>> + Send + 'a
    where
        Val: SerializeRow + Send + Sync,
        Qe: QueryExecutor<M> + Send,
    {
        async move {
            let res = session
                .execute_unpaged(query.query_string, query.values)
                .await
                .map_err(|e| DbError::ExecutionError(query.query_string, e))?
                .into_rows_result()
                .map_err(|e| DbError::IntoRowsResultError(query.query_string, e))?;

            res.first_row::<M>().map_err(|e| match e {
                FirstRowError::RowsEmpty => DbError::NotFoundError(query.query_string),
                FirstRowError::DeserializationFailed(de) => {
                    DbError::DeserializationError(query.query_string, Box::new(de))
                },
                FirstRowError::TypeCheckFailed(te) => {
                    DbError::DeserializationError(query.query_string, Box::new(te))
                },
            })
        }
    }
}

impl<M: BaseModel + Send + Sync + 'static> QueryExecutor<M> for OptionalRow {
    fn execute<'a, Val, Qe>(
        query: ScyllaQuery<'a, Val, M, Qe>,
        session: &'a CachingSession,
    ) -> impl std::future::Future<Output = Result<Option<M>, DbError>> + Send + 'a
    where
        Val: SerializeRow + Send + Sync,
        Qe: QueryExecutor<M> + Send,
    {
        async move {
            let res = session
                .execute_unpaged(query.query_string, query.values)
                .await
                .map_err(|e| DbError::ExecutionError(query.query_string, e))?
                .into_rows_result()
                .map_err(|e| DbError::IntoRowsResultError(query.query_string, e))?;

            match res.first_row::<M>() {
                Ok(row) => Ok(Some(row)),
                Err(FirstRowError::RowsEmpty) => Ok(None),
                Err(FirstRowError::DeserializationFailed(de)) => {
                    Err(DbError::DeserializationError(query.query_string, Box::new(de)))
                },
                Err(FirstRowError::TypeCheckFailed(te)) => {
                    Err(DbError::DeserializationError(query.query_string, Box::new(te)))
                },
            }
        }
    }
}

impl<M: BaseModel + Send + Sync + 'static> QueryExecutor<M> for Stream {
    fn execute<'a, Val, Qe>(
        query: ScyllaQuery<'a, Val, M, Qe>,
        session: &'a CachingSession,
    ) -> impl std::future::Future<Output = Result<Vec<M>, DbError>> + Send + 'a
    where
        Val: SerializeRow + Send + Sync,
        Qe: QueryExecutor<M> + Send,
    {
        use futures::StreamExt;

        async move {
            let qs = query.query_string;
            let typed_stream = session
                .execute_iter(query.query_string, query.values)
                .await
                .map_err(|e| DbError::PagerExecutionError(qs, Box::new(e)))?
                .rows_stream::<M>()
                .map_err(|e| DbError::RowsError(qs, e.into()))?;

            // Collect the async stream into a Vec
            futures::pin_mut!(typed_stream);
            let mut results = Vec::new();
            while let Some(item) = typed_stream.next().await {
                results.push(item.map_err(|e| DbError::DeserializationError(qs, Box::new(e)))?);
            }
            Ok(results)
        }
    }
}

impl<M: BaseModel + Send + Sync + 'static> QueryExecutor<M> for Paged {
    fn execute<'a, Val, Qe>(
        query: ScyllaQuery<'a, Val, M, Qe>,
        session: &'a CachingSession,
    ) -> impl std::future::Future<Output = Result<(Vec<M>, PagingStateResponse), DbError>> + Send + 'a
    where
        Val: SerializeRow + Send + Sync,
        Qe: QueryExecutor<M> + Send,
    {
        async move {
            let qs = query.query_string;
            let paging_state = query.paging_state.unwrap_or_else(PagingState::start);
            let (result, psr) = session
                .execute_single_page(qs, query.values, paging_state)
                .await
                .map_err(|e| DbError::ExecutionError(qs, e))?;

            let rows_result =
                result.into_rows_result().map_err(|e| DbError::IntoRowsResultError(qs, e))?;

            let models: Result<Vec<M>, DbError> = rows_result
                .rows::<M>()
                .map_err(|e| DbError::RowsError(qs, e))?
                .map(|r| r.map_err(|e| DbError::DeserializationError(qs, Box::new(e))))
                .collect();

            Ok((models?, psr))
        }
    }
}

impl<M: BaseModel + Send + Sync + 'static> QueryExecutor<M> for Mutation {
    fn execute<'a, Val, Qe>(
        query: ScyllaQuery<'a, Val, M, Qe>,
        session: &'a CachingSession,
    ) -> impl std::future::Future<Output = Result<(), DbError>> + Send + 'a
    where
        Val: SerializeRow + Send + Sync,
        Qe: QueryExecutor<M> + Send,
    {
        async move {
            session
                .execute_unpaged(query.query_string, query.values)
                .await
                .map_err(|e| DbError::ExecutionError(query.query_string, e))?;
            Ok(())
        }
    }
}

// ── QueryValue ────────────────────────────────────────────────────────────────

/// Holds serializable query values with minimal copying.
pub enum QueryValue<'a, Val: SerializeRow> {
    Owned(Val),
    Ref(&'a Val),
    Empty,
}

impl<Val: SerializeRow> SerializeRow for QueryValue<'_, Val> {
    fn serialize(
        &self,
        ctx: &scylla::serialize::row::RowSerializationContext<'_>,
        writer: &mut scylla::serialize::writers::RowWriter,
    ) -> Result<(), scylla::serialize::SerializationError> {
        match self {
            QueryValue::Owned(v) => v.serialize(ctx, writer),
            QueryValue::Ref(v) => v.serialize(ctx, writer),
            QueryValue::Empty => Ok(()),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            QueryValue::Owned(v) => v.is_empty(),
            QueryValue::Ref(v) => v.is_empty(),
            QueryValue::Empty => true,
        }
    }
}

// ── ScyllaQuery ───────────────────────────────────────────────────────────────

/// Builder for a CQL query that can be executed against a `CachingSession`.
///
/// Type parameters:
/// - `'a`  — lifetime of any borrowed values
/// - `Val` — serializable value tuple
/// - `M`   — the model row struct being read or written
/// - `Qe`  — execution strategy marker (`RowResult`, `OptionalRow`, …)
pub struct ScyllaQuery<'a, Val: SerializeRow, M: BaseModel, Qe: QueryExecutor<M>> {
    pub query_string: &'static str,
    pub values: QueryValue<'a, Val>,
    pub inner: Statement,
    pub paging_state: Option<PagingState>,
    _model: std::marker::PhantomData<M>,
    _qe: std::marker::PhantomData<Qe>,
}

impl<'a, Val: SerializeRow, M: BaseModel, Qe: QueryExecutor<M>> ScyllaQuery<'a, Val, M, Qe> {
    pub fn new(query_string: &'static str, values: QueryValue<'a, Val>) -> Self {
        Self {
            query_string,
            values,
            inner: Statement::new(query_string),
            paging_state: None,
            _model: std::marker::PhantomData,
            _qe: std::marker::PhantomData,
        }
    }

    pub fn page_size(mut self, page_size: i32) -> Self {
        self.inner.set_page_size(page_size);
        self
    }

    pub fn consistency(mut self, consistency: scylla::statement::Consistency) -> Self {
        self.inner.set_consistency(consistency);
        self
    }

    pub fn idempotent(mut self, is_idempotent: bool) -> Self {
        self.inner.set_is_idempotent(is_idempotent);
        self
    }

    pub fn tracing(mut self, enabled: bool) -> Self {
        self.inner.set_tracing(enabled);
        self
    }

    pub fn timestamp(mut self, timestamp: Option<i64>) -> Self {
        self.inner.set_timestamp(timestamp);
        self
    }

    pub fn timeout(mut self, timeout: Option<std::time::Duration>) -> Self {
        self.inner.set_request_timeout(timeout);
        self
    }

    pub fn paging_state(mut self, paging_state: PagingState) -> Self {
        self.paging_state = Some(paging_state);
        self
    }

    /// Execute the query and return the typed output.
    pub async fn execute(self, session: &CachingSession) -> Result<Qe::Output, DbError>
    where
        Val: Send + Sync,
        M: Send + Sync + 'static,
        Qe: Send,
    {
        Qe::execute(self, session).await
    }
}
