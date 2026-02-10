use crate::{
    application::dto::{RolePermissions, RoleResponse},
    domain::{
        repositories::user_repository::UserRepository,
        value_objects::{UserId, UserRole},
    },
};
use std::sync::Arc;

/// Use case for getting a user's role
pub struct GetUserRoleUseCase<R: UserRepository> {
    user_repo: Arc<R>,
}

impl<R: UserRepository> GetUserRoleUseCase<R> {
    pub fn new(user_repo: Arc<R>) -> Self {
        Self { user_repo }
    }

    pub async fn execute(&self, user_id: &str) -> Result<RoleResponse, GetRoleError> {
        // Parse user ID
        let user_id = UserId::from_string(user_id).map_err(|_| GetRoleError::InvalidUserId)?;

        // Find user
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(|e| GetRoleError::Repository(e.to_string()))?
            .ok_or(GetRoleError::UserNotFound)?;

        // Build response
        Ok(RoleResponse {
            user_id: user.id.to_string(),
            email: user.email.as_str().to_string(),
            role: user.role.to_string(),
            permissions: RolePermissions {
                can_read: user.role.can_read(),
                can_write: user.role.can_write(),
                can_delete: user.role.can_delete(),
            },
        })
    }
}

/// Use case for updating a user's role
pub struct UpdateUserRoleUseCase<R: UserRepository> {
    user_repo: Arc<R>,
}

impl<R: UserRepository> UpdateUserRoleUseCase<R> {
    pub fn new(user_repo: Arc<R>) -> Self {
        Self { user_repo }
    }

    pub async fn execute(
        &self,
        user_id: &str,
        new_role: &str,
    ) -> Result<RoleResponse, UpdateRoleError> {
        // Parse user ID
        let user_id = UserId::from_string(user_id).map_err(|_| UpdateRoleError::InvalidUserId)?;

        // Parse and validate role
        let role = UserRole::parse(new_role)
            .ok_or_else(|| UpdateRoleError::InvalidRole(new_role.to_string()))?;

        // Find user
        let mut user = self
            .user_repo
            .find_by_id(user_id)
            .await
            .map_err(|e| UpdateRoleError::Repository(e.to_string()))?
            .ok_or(UpdateRoleError::UserNotFound)?;

        // Update role
        user.role = role;

        // Save user
        let updated_user = self
            .user_repo
            .save(&user)
            .await
            .map_err(|e| UpdateRoleError::Repository(e.to_string()))?;

        // Build response
        Ok(RoleResponse {
            user_id: updated_user.id.to_string(),
            email: updated_user.email.as_str().to_string(),
            role: updated_user.role.to_string(),
            permissions: RolePermissions {
                can_read: updated_user.role.can_read(),
                can_write: updated_user.role.can_write(),
                can_delete: updated_user.role.can_delete(),
            },
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetRoleError {
    #[error("Invalid user ID")]
    InvalidUserId,
    #[error("User not found")]
    UserNotFound,
    #[error("Repository error: {0}")]
    Repository(String),
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateRoleError {
    #[error("Invalid user ID")]
    InvalidUserId,
    #[error("Invalid role: {0}. Must be 'admin', 'editor', or 'viewer'")]
    InvalidRole(String),
    #[error("User not found")]
    UserNotFound,
    #[error("Repository error: {0}")]
    Repository(String),
}
