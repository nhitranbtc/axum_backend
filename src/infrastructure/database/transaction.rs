use diesel::result::Error as DieselError;
use diesel_async::pooled_connection::deadpool::Object;
use diesel_async::AsyncPgConnection;
use futures::future::BoxFuture;

/// Connection type used in the repository
pub type Conn = Object<AsyncPgConnection>;

/// Trait for executing database operations within a transaction
#[async_trait::async_trait]
pub trait TransactionalRepository {
    async fn with_transaction<F, R, E>(&self, f: F) -> Result<R, E>
    where
        F: for<'a> FnOnce(&'a mut AsyncPgConnection) -> BoxFuture<'a, Result<R, E>> + Send,
        E: From<DieselError> + Send,
        R: Send;
}
