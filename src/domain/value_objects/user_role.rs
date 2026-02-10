use serde::{Deserialize, Serialize};
use std::fmt;

/// User roles in the system
///
/// Roles define what actions a user can perform:
/// - Admin: Full access (read, write, delete)
/// - Editor: Can read and write, but cannot delete
/// - Viewer: Can only read data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    /// Administrator - Full access to all operations
    Admin,
    /// Editor - Can read and write, but cannot delete
    Editor,
    /// Viewer - Read-only access
    #[default]
    Viewer,
}

impl UserRole {
    /// Check if this role can read data
    pub fn can_read(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::Editor | UserRole::Viewer)
    }

    /// Check if this role can write (create/update) data
    pub fn can_write(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::Editor)
    }

    /// Check if this role can delete data
    pub fn can_delete(&self) -> bool {
        matches!(self, UserRole::Admin)
    }

    /// Get all available roles
    pub fn all() -> Vec<UserRole> {
        vec![UserRole::Admin, UserRole::Editor, UserRole::Viewer]
    }

    /// Parse role from string
    pub fn parse(s: &str) -> Option<UserRole> {
        match s.to_lowercase().as_str() {
            "admin" => Some(UserRole::Admin),
            "editor" => Some(UserRole::Editor),
            "viewer" => Some(UserRole::Viewer),
            _ => None,
        }
    }
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::Editor => write!(f, "editor"),
            UserRole::Viewer => write!(f, "viewer"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_permissions() {
        let admin = UserRole::Admin;
        assert!(admin.can_read());
        assert!(admin.can_write());
        assert!(admin.can_delete());
    }

    #[test]
    fn test_editor_permissions() {
        let editor = UserRole::Editor;
        assert!(editor.can_read());
        assert!(editor.can_write());
        assert!(!editor.can_delete());
    }

    #[test]
    fn test_viewer_permissions() {
        let viewer = UserRole::Viewer;
        assert!(viewer.can_read());
        assert!(!viewer.can_write());
        assert!(!viewer.can_delete());
    }

    #[test]
    fn test_role_from_str() {
        assert_eq!(UserRole::parse("admin"), Some(UserRole::Admin));
        assert_eq!(UserRole::parse("ADMIN"), Some(UserRole::Admin));
        assert_eq!(UserRole::parse("editor"), Some(UserRole::Editor));
        assert_eq!(UserRole::parse("viewer"), Some(UserRole::Viewer));
        assert_eq!(UserRole::parse("invalid"), None);
    }
}
