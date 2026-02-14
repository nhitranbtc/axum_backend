use std::fmt;

/// Subject version for versioned event streams
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubjectVersion {
    V1,
    V2,
}

impl fmt::Display for SubjectVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubjectVersion::V1 => write!(f, "v1"),
            SubjectVersion::V2 => write!(f, "v2"),
        }
    }
}

/// User event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserEventType {
    Created,
    Updated,
    Deleted,
}

impl fmt::Display for UserEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserEventType::Created => write!(f, "created"),
            UserEventType::Updated => write!(f, "updated"),
            UserEventType::Deleted => write!(f, "deleted"),
        }
    }
}

/// Subject builder for user events
/// Pattern: {env}.users.{version}.{event_type}
pub struct UserSubject;

impl UserSubject {
    /// Build a specific user event subject
    /// Example: prod.users.v1.created
    pub fn build(env: &str, version: SubjectVersion, event_type: UserEventType) -> String {
        format!("{}.users.{}.{}", env, version, event_type)
    }

    /// Build a wildcard subject for all events of a specific version
    /// Example: prod.users.v1.*
    pub fn build_version_wildcard(env: &str, version: SubjectVersion) -> String {
        format!("{}.users.{}.*", env, version)
    }

    /// Build a wildcard subject for all user events across all versions
    /// Example: prod.users.>
    pub fn build_all_wildcard(env: &str) -> String {
        format!("{}.users.>", env)
    }

    /// Build a subject for a specific event type across all versions
    /// Example: prod.users.*.created
    pub fn build_event_wildcard(env: &str, event_type: UserEventType) -> String {
        format!("{}.users.*.{}", env, event_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_specific_subject() {
        let subject = UserSubject::build("prod", SubjectVersion::V1, UserEventType::Created);
        assert_eq!(subject, "prod.users.v1.created");

        let subject = UserSubject::build("dev", SubjectVersion::V2, UserEventType::Updated);
        assert_eq!(subject, "dev.users.v2.updated");
    }

    #[test]
    fn test_build_version_wildcard() {
        let subject = UserSubject::build_version_wildcard("prod", SubjectVersion::V1);
        assert_eq!(subject, "prod.users.v1.*");

        let subject = UserSubject::build_version_wildcard("staging", SubjectVersion::V2);
        assert_eq!(subject, "staging.users.v2.*");
    }

    #[test]
    fn test_build_all_wildcard() {
        let subject = UserSubject::build_all_wildcard("prod");
        assert_eq!(subject, "prod.users.>");
    }

    #[test]
    fn test_build_event_wildcard() {
        let subject = UserSubject::build_event_wildcard("prod", UserEventType::Created);
        assert_eq!(subject, "prod.users.*.created");
    }

    #[test]
    fn test_version_display() {
        assert_eq!(SubjectVersion::V1.to_string(), "v1");
        assert_eq!(SubjectVersion::V2.to_string(), "v2");
    }

    #[test]
    fn test_event_type_display() {
        assert_eq!(UserEventType::Created.to_string(), "created");
        assert_eq!(UserEventType::Updated.to_string(), "updated");
        assert_eq!(UserEventType::Deleted.to_string(), "deleted");
    }
}
