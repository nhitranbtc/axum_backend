use crate::domain::entities::User;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

/// DTO for creating a new user
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "email": "test_user01@mail.com",
    "name": "User01"
}))]
pub struct CreateUserDto {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 1, max = 255))]
    pub name: String,
}

/// DTO for updating a user
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "name": "Updated User01"
}))]
pub struct UpdateUserDto {
    #[validate(length(min = 1, max = 255))]
    pub name: Option<String>,
}

/// DTO for user response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserResponseDto {
    pub id: String,
    pub email: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<User> for UserResponseDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            email: user.email.to_string(),
            name: user.name,
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
        }
    }
}

impl From<&User> for UserResponseDto {
    fn from(user: &User) -> Self {
        Self {
            id: user.id.to_string(),
            email: user.email.to_string(),
            name: user.name.clone(),
            created_at: user.created_at.to_rfc3339(),
            updated_at: user.updated_at.to_rfc3339(),
        }
    }
}
