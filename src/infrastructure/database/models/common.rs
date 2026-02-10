/// Common database model utilities and traits
use chrono::{DateTime, Utc};

/// Trait for models with timestamps
pub trait Timestamped {
    fn created_at(&self) -> DateTime<Utc>;
    fn updated_at(&self) -> DateTime<Utc>;
}

/// Trait for models with soft delete
pub trait SoftDeletable {
    fn deleted_at(&self) -> Option<DateTime<Utc>>;
    fn is_deleted(&self) -> bool {
        self.deleted_at().is_some()
    }
}

/// Trait for models with UUID primary key
pub trait HasUuid {
    fn id(&self) -> uuid::Uuid;
}
