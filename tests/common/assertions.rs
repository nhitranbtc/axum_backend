#![allow(dead_code)]

use serde_json::Value;

/// Assert response is successful
pub fn assert_success(response: &Value) {
    assert_eq!(
        response["success"].as_bool().unwrap_or(false),
        true,
        "Response was not successful: {:?}",
        response
    );
}

/// Assert response has error
pub fn assert_error(response: &Value) {
    assert_eq!(
        response["success"].as_bool().unwrap_or(true),
        false,
        "Response should have failed: {:?}",
        response
    );
}
