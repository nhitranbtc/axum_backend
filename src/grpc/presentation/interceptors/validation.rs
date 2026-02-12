use tonic::Status;
use uuid::Uuid;

/// Validates that a string is a valid UUID
pub fn validate_uuid(uuid_str: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(uuid_str)
        .map_err(|e| Status::invalid_argument(format!("Invalid UUID '{}': {}", uuid_str, e)))
}

/// Validates that a string is not empty
pub fn validate_not_empty(value: &str, field_name: &str) -> Result<(), Status> {
    if value.trim().is_empty() {
        return Err(Status::invalid_argument(format!("{} cannot be empty", field_name)));
    }
    Ok(())
}

/// Validates that a value is within a range
pub fn validate_range<T: PartialOrd + std::fmt::Display>(
    value: T,
    min: T,
    max: T,
    field_name: &str,
) -> Result<(), Status> {
    if value < min || value > max {
        return Err(Status::invalid_argument(format!(
            "{} must be between {} and {}, got {}",
            field_name, min, max, value
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_uuid_valid() {
        let result = validate_uuid("771ddd71-1e10-46e5-a4d4-bc674e6d6a53");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_uuid_invalid() {
        let result = validate_uuid("not-a-uuid");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_not_empty_valid() {
        let result = validate_not_empty("hello", "name");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_not_empty_invalid() {
        let result = validate_not_empty("  ", "name");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_range_valid() {
        let result = validate_range(5, 1, 10, "value");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_range_invalid() {
        let result = validate_range(15, 1, 10, "value");
        assert!(result.is_err());
    }
}
