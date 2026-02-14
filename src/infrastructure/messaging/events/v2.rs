use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::traits::Event;

/// User created event (v2)
/// Enhanced version with additional metadata and observability fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserCreatedEventV2 {
    pub user_id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub is_active: bool,
    pub is_email_verified: bool,
    pub created_at: DateTime<Utc>,
    /// Additional metadata for observability and tracking
    pub metadata: HashMap<String, String>,
}

impl UserCreatedEventV2 {
    pub fn new(
        user_id: String,
        email: String,
        name: String,
        role: String,
        is_active: bool,
        is_email_verified: bool,
    ) -> Self {
        Self {
            user_id,
            email,
            name,
            role,
            is_active,
            is_email_verified,
            created_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata entry
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

// Implement Event trait for automatic serialization
impl Event for UserCreatedEventV2 {}

/// Field change tracking for v2 events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldChange {
    pub previous_value: Option<String>,
    pub new_value: Option<String>,
}

impl FieldChange {
    pub fn new(previous_value: Option<String>, new_value: Option<String>) -> Self {
        Self {
            previous_value,
            new_value,
        }
    }
}

/// User updated event (v2)
/// Enhanced version with previous/new value tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserUpdatedEventV2 {
    pub user_id: String,
    pub name_change: Option<FieldChange>,
    pub email_change: Option<FieldChange>,
    pub role_change: Option<FieldChange>,
    pub updated_at: DateTime<Utc>,
    pub updated_by: Option<String>,
    /// Additional metadata for observability
    pub metadata: HashMap<String, String>,
}

impl UserUpdatedEventV2 {
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            name_change: None,
            email_change: None,
            role_change: None,
            updated_at: Utc::now(),
            updated_by: None,
            metadata: HashMap::new(),
        }
    }

    /// Set name change
    pub fn with_name_change(mut self, previous: Option<String>, new: Option<String>) -> Self {
        self.name_change = Some(FieldChange::new(previous, new));
        self
    }

    /// Set email change
    pub fn with_email_change(mut self, previous: Option<String>, new: Option<String>) -> Self {
        self.email_change = Some(FieldChange::new(previous, new));
        self
    }

    /// Set role change
    pub fn with_role_change(mut self, previous: Option<String>, new: Option<String>) -> Self {
        self.role_change = Some(FieldChange::new(previous, new));
        self
    }

    /// Set who updated the user
    pub fn with_updated_by(mut self, updated_by: String) -> Self {
        self.updated_by = Some(updated_by);
        self
    }

    /// Add metadata entry
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

// Implement Event trait for automatic serialization
impl Event for UserUpdatedEventV2 {}

/// User deleted event (v2)
/// Enhanced version with deletion reason and metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserDeletedEventV2 {
    pub user_id: String,
    pub deleted_at: DateTime<Utc>,
    pub deleted_by: Option<String>,
    pub reason: Option<String>,
    /// Additional metadata for audit trail
    pub metadata: HashMap<String, String>,
}

impl UserDeletedEventV2 {
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            deleted_at: Utc::now(),
            deleted_by: None,
            reason: None,
            metadata: HashMap::new(),
        }
    }

    /// Set who deleted the user
    pub fn with_deleted_by(mut self, deleted_by: String) -> Self {
        self.deleted_by = Some(deleted_by);
        self
    }

    /// Set deletion reason
    pub fn with_reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }

    /// Add metadata entry
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

// Implement Event trait for automatic serialization
impl Event for UserDeletedEventV2 {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_created_event_v2_serialization() {
        let event = UserCreatedEventV2::new(
            "user-123".to_string(),
            "test@example.com".to_string(),
            "John Doe".to_string(),
            "user".to_string(),
            true,
            false,
        )
        .with_metadata("source".to_string(), "api".to_string());

        let bytes = event.to_bytes().unwrap();
        let deserialized = UserCreatedEventV2::from_bytes(&bytes).unwrap();

        assert_eq!(event.user_id, deserialized.user_id);
        assert_eq!(event.email, deserialized.email);
        assert_eq!(event.name, deserialized.name);
        assert_eq!(event.role, deserialized.role);
        assert_eq!(event.metadata, deserialized.metadata);
    }

    #[test]
    fn test_user_updated_event_v2_serialization() {
        let event = UserUpdatedEventV2::new("user-123".to_string())
            .with_name_change(
                Some("John Doe".to_string()),
                Some("Jane Doe".to_string()),
            )
            .with_updated_by("admin-456".to_string())
            .with_metadata("reason".to_string(), "name_correction".to_string());

        let bytes = event.to_bytes().unwrap();
        let deserialized = UserUpdatedEventV2::from_bytes(&bytes).unwrap();

        assert_eq!(event.user_id, deserialized.user_id);
        assert_eq!(event.name_change, deserialized.name_change);
        assert_eq!(event.updated_by, deserialized.updated_by);
        assert_eq!(event.metadata, deserialized.metadata);
    }

    #[test]
    fn test_user_deleted_event_v2_serialization() {
        let event = UserDeletedEventV2::new("user-123".to_string())
            .with_deleted_by("admin-456".to_string())
            .with_reason("GDPR request".to_string())
            .with_metadata("ticket_id".to_string(), "TICKET-789".to_string());

        let bytes = event.to_bytes().unwrap();
        let deserialized = UserDeletedEventV2::from_bytes(&bytes).unwrap();

        assert_eq!(event.user_id, deserialized.user_id);
        assert_eq!(event.deleted_by, deserialized.deleted_by);
        assert_eq!(event.reason, deserialized.reason);
        assert_eq!(event.metadata, deserialized.metadata);
    }

    #[test]
    fn test_field_change() {
        let change = FieldChange::new(
            Some("old_value".to_string()),
            Some("new_value".to_string()),
        );

        assert_eq!(change.previous_value, Some("old_value".to_string()));
        assert_eq!(change.new_value, Some("new_value".to_string()));
    }
}
