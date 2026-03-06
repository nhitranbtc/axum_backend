use scylla::deserialize::row::DeserializeRow;
use scylla::serialize::row::SerializeRow;

/// Minimal model metadata required for query building.
///
/// Every row struct in the scylla layer must implement this. It declares:
/// - The primary key tuple type (for PK-based lookups)
/// - The partition key tuple type (for partition-scan queries)
/// - A set of standard query string constants
///
/// Mirrors charybdis `BaseModel`.
pub trait BaseModel:
    SerializeRow + for<'frame, 'metadata> DeserializeRow<'frame, 'metadata> + Send + Sync
{
    /// Tuple type of the primary key columns (partition + clustering).
    type PrimaryKey: SerializeRow + Send + Sync;

    /// Tuple type of the partition key columns only.
    type PartitionKey: SerializeRow + Send + Sync;

    /// Table name as it appears in CQL.
    const TABLE_NAME: &'static str;

    /// `SELECT … FROM <table>` (all rows, no limit).
    const FIND_ALL_QUERY: &'static str;

    /// `SELECT … FROM <table> WHERE <pk_cols> = ?`
    const FIND_BY_PRIMARY_KEY_QUERY: &'static str;

    /// `SELECT … FROM <table> WHERE <partition_cols> = ?`
    const FIND_BY_PARTITION_KEY_QUERY: &'static str;

    /// Extract a tuple containing this row's primary key values.
    fn primary_key_values(&self) -> Self::PrimaryKey;

    /// Extract a tuple containing only this row's partition key values.
    fn partition_key_values(&self) -> Self::PartitionKey;
}

/// Full CRUD model — extends `BaseModel` with write operation constants.
///
/// Mirrors charybdis `Model`. Every row struct that needs insert/update/delete
/// operations must implement this trait.
pub trait Model: BaseModel {
    /// Full INSERT INTO … VALUES … statement.
    const INSERT_QUERY: &'static str;

    /// INSERT INTO … IF NOT EXISTS
    const INSERT_IF_NOT_EXISTS_QUERY: &'static str;

    /// Full UPDATE SET … WHERE <pk> = ?
    const UPDATE_QUERY: &'static str;

    /// DELETE FROM <table> WHERE <pk_cols> = ?
    const DELETE_QUERY: &'static str;

    /// DELETE FROM <table> WHERE <partition_cols> = ?
    const DELETE_BY_PARTITION_KEY_QUERY: &'static str;
}
