use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::traits::Event;

/// User created event (v1)
/// Basic version with essential fields only
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserCreatedEventV1 {
    pub user_id: String,
    pub email: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

impl UserCreatedEventV1 {
    pub fn new(user_id: String, email: String, name: String) -> Self {
        Self {
            user_id,
            email,
            name,
            created_at: Utc::now(),
        }
    }
}

// Implement Event trait for automatic serialization
impl Event for UserCreatedEventV1 {}

/// User updated event (v1)
/// Basic version with changed fields
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserUpdatedEventV1 {
    pub user_id: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl UserUpdatedEventV1 {
    pub fn new(user_id: String, name: Option<String>, email: Option<String>) -> Self {
        Self {
            user_id,
            name,
            email,
            updated_at: Utc::now(),
        }
    }
}

// Implement Event trait for automatic serialization
impl Event for UserUpdatedEventV1 {}

/// User deleted event (v1)
/// Basic version with user ID only
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserDeletedEventV1 {
    pub user_id: String,
    pub deleted_at: DateTime<Utc>,
}

impl UserDeletedEventV1 {
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            deleted_at: Utc::now(),
        }
    }
}

// Implement Event trait for automatic serialization
impl Event for UserDeletedEventV1 {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_created_event_v1_serialization() {
        let event = UserCreatedEventV1::new(
            "user-123".to_string(),
            "test@example.com".to_string(),
            "John Doe".to_string(),
        );

        let bytes = event.to_bytes().unwrap();
        let deserialized = UserCreatedEventV1::from_bytes(&bytes).unwrap();

        assert_eq!(event.user_id, deserialized.user_id);
        assert_eq!(event.email, deserialized.email);
        assert_eq!(event.name, deserialized.name);
    }

    #[test]
    fn test_user_updated_event_v1_serialization() {
        let event = UserUpdatedEventV1::new(
            "user-123".to_string(),
            Some("Jane Doe".to_string()),
            None,
        );

        let bytes = event.to_bytes().unwrap();
        let deserialized = UserUpdatedEventV1::from_bytes(&bytes).unwrap();

        assert_eq!(event.user_id, deserialized.user_id);
        assert_eq!(event.name, deserialized.name);
        assert_eq!(event.email, deserialized.email);
    }

    #[test]
    fn test_user_deleted_event_v1_serialization() {
        let event = UserDeletedEventV1::new("user-123".to_string());

        let bytes = event.to_bytes().unwrap();
        let deserialized = UserDeletedEventV1::from_bytes(&bytes).unwrap();

        assert_eq!(event.user_id, deserialized.user_id);
    }
}
