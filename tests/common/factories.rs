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
    // Note: In ScyllaDB, users table uses user_id as PK. If we only have email,
    // we must query the user_id first (requires ALLOW FILTERING or an index)
    // Then delete by user_id.

    let query = "SELECT user_id FROM users WHERE email = ? ALLOW FILTERING";
    let values = (email,);

    if let Ok(res) = pool.session().query_unpaged(query, values).await {
        if let Ok(rows_res) = res.into_rows_result() {
            if let Ok(mut rows) = rows_res.rows::<(uuid::Uuid,)>() {
                if let Some(Ok((user_id,))) = rows.next() {
                    let delete_query = "DELETE FROM users WHERE user_id = ?";
                    let _ = pool.session().query_unpaged(delete_query, (user_id,)).await;
                }
            }
        }
    }
}
