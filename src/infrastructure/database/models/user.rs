use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::infrastructure::database::schema::users;

/// Database model for User entity
///
/// This represents the database table structure and is used by Diesel ORM.
/// It's separate from the domain `User` entity to maintain clean architecture.
#[derive(Debug, Clone, Queryable, Selectable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserModel {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub password_hash: Option<String>,
    pub role: String,
    pub is_active: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub confirmation_code: Option<String>,
    pub confirmation_code_expires_at: Option<DateTime<Utc>>,
    pub email_verified: bool,
}

impl UserModel {
    /// Create a new user model for database insertion
    pub fn new(
        id: Uuid,
        email: String,
        name: String,
        password_hash: Option<String>,
        role: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            email,
            name,
            password_hash,
            role,
            is_active: false, // Default inactive until verified
            last_login: None,
            created_at: now,
            updated_at: now,
            confirmation_code: None,
            confirmation_code_expires_at: None,
            email_verified: false,
        }
    }

    /// Update the timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }
}

// Backward compatibility alias (will be deprecated)
#[deprecated(since = "0.2.0", note = "Use `UserModel` instead")]
pub type DbUser = UserModel;
