use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request to update a user's role
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateRoleRequest {
    /// New role for the user. Must be one of: admin, editor, viewer
    #[schema(example = "editor")]
    pub role: String,
}

/// Response containing user role information
#[derive(Debug, Serialize, ToSchema)]
pub struct RoleResponse {
    /// User's unique identifier
    pub user_id: String,
    /// User's email address
    pub email: String,
    /// User's current role
    #[schema(example = "editor")]
    pub role: String,
    /// Permissions granted by this role
    pub permissions: RolePermissions,
}

/// Role permissions breakdown
#[derive(Debug, Serialize, ToSchema)]
pub struct RolePermissions {
    /// Can read data
    pub can_read: bool,
    /// Can write/modify data
    pub can_write: bool,
    /// Can delete data
    pub can_delete: bool,
}
