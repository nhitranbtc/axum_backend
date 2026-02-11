#![allow(dead_code)]

use axum_backend::infrastructure::database::DbPool;

/// Generate a unique test email
pub fn unique_email(prefix: &str) -> String {
    format!(
        "{}_{}_{}@test.com",
        prefix,
        uuid::Uuid::new_v4(),
        chrono::Utc::now().timestamp()
    )
}

/// Generate a unique test name
pub fn unique_name(prefix: &str) -> String {
    format!("{} {}", prefix, uuid::Uuid::new_v4())
}

/// Test password
pub const TEST_PASSWORD: &str = "TestPassword@123";

/// Clean up test data (optional, for specific tests)
pub async fn cleanup_test_user(pool: &DbPool, email: &str) {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    let mut conn = pool.get().await.expect("Failed to get connection");

    diesel::delete(
        axum_backend::infrastructure::database::schema::users::table
            .filter(axum_backend::infrastructure::database::schema::users::email.eq(email)),
    )
    .execute(&mut conn)
    .await
    .ok();
}
