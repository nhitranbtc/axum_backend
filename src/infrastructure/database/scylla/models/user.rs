use chrono::{DateTime, Utc};
use uuid::Uuid;

/// ScyllaDB row for the `users` table.
#[derive(Debug, Clone)]
pub struct UserRow {
    pub user_id: Uuid,
    pub email: String,
    pub name: String,
    pub password_hash: Option<String>,
    pub role: String,
    pub is_active: bool,
    pub email_verified: bool,
    pub confirmation_code: Option<String>,
    pub confirmation_code_expires_at: Option<DateTime<Utc>>,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserRow {
    pub fn new(
        user_id: Uuid,
        email: String,
        name: String,
        password_hash: Option<String>,
        role: String,
        is_active: bool,
        email_verified: bool,
        confirmation_code: Option<String>,
        confirmation_code_expires_at: Option<DateTime<Utc>>,
        last_login: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            user_id,
            email,
            name,
            password_hash,
            role,
            is_active,
            email_verified,
            confirmation_code,
            confirmation_code_expires_at,
            last_login,
            created_at,
            updated_at,
        }
    }
}
