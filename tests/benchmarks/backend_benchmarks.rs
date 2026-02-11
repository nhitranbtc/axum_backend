use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tokio::runtime::Runtime;

// --- API Benchmarks ---
const BASE_URL: &str = "http://127.0.0.1:3000";

fn bench_api_register(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let client = Client::new();

    let mut group = c.benchmark_group("api_auth_register");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(50); // Reduced for speed in CI

    group.bench_function("register_user", |b| {
        b.to_async(&rt).iter(|| async {
            let email = format!("bench_{}@test.com", rand::random::<u32>());
            let response = client
                .post(format!("{}/api/auth/register", BASE_URL))
                .json(&json!({
                    "email": email,
                    "name": "Bench User",
                    "password": "BenchPass@123"
                }))
                .send()
                .await;
            black_box(response)
        });
    });
    group.finish();
}

fn bench_api_login(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let client = Client::new();

    // Setup: Create a test user
    rt.block_on(async {
        let _ = client
            .post(format!("{}/api/auth/register", BASE_URL))
            .json(&json!({
                "email": "bench_login@test.com",
                "name": "Login Bench",
                "password": "BenchPass@123"
            }))
            .send()
            .await;
    });

    let mut group = c.benchmark_group("api_auth_login");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(50);

    group.bench_function("login_user", |b| {
        b.to_async(&rt).iter(|| async {
            let response = client
                .post(format!("{}/api/auth/login", BASE_URL))
                .json(&json!({
                    "email": "bench_login@test.com",
                    "password": "BenchPass@123"
                }))
                .send()
                .await;
            black_box(response)
        });
    });
    group.finish();
}

fn bench_api_list_users(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let client = Client::new();

    // Setup: Get auth token
    let token = rt.block_on(async {
        // Register first to ensure user exists
        let _ = client
            .post(format!("{}/api/auth/register", BASE_URL))
            .json(&json!({
                "email": "bench_list@test.com",
                "name": "List Bench",
                "password": "BenchPass@123"
            }))
            .send()
            .await;

        let response = client
            .post(format!("{}/api/auth/login", BASE_URL))
            .json(&json!({
                "email": "bench_list@test.com",
                "password": "BenchPass@123"
            }))
            .send()
            .await
            .expect("Login failed");

        let body: serde_json::Value = response.json().await.unwrap();
        body["data"]["access_token"].as_str().unwrap_or("").to_string()
    });

    if token.is_empty() {
        eprintln!("Skipping list_users benchmark due to missing token");
        return;
    }

    let mut group = c.benchmark_group("api_users_list");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(50);

    for page_size in [10, 50].iter() {
        group.throughput(Throughput::Elements(*page_size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(page_size), page_size, |b, &size| {
            b.to_async(&rt).iter(|| {
                let client = client.clone();
                let token = token.clone();
                async move {
                    let response = client
                        .get(format!("{}/api/users?page=1&page_size={}", BASE_URL, size))
                        .header("Authorization", format!("Bearer {}", token))
                        .send()
                        .await;
                    black_box(response)
                }
            });
        });
    }
    group.finish();
}

fn bench_api_health_check(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let client = Client::new();

    let mut group = c.benchmark_group("api_health_check");
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(100);

    group.bench_function("health_endpoint", |b| {
        b.to_async(&rt).iter(|| async {
            let response = client.get(format!("{}/health", BASE_URL)).send().await;
            black_box(response)
        });
    });
    group.finish();
}

// --- Core Benchmarks (No external dependencies preferred) ---

fn bench_core_jwt_creation(c: &mut Criterion) {
    use axum_backend::shared::utils::jwt::JwtManager;
    use uuid::Uuid;

    let jwt_manager = JwtManager::new(
        "benchmark-secret-key-must-be-at-least-32-bytes-long".to_string(),
        3600,
        604800,
        "benchmark-issuer".to_string(),
        "benchmark-audience".to_string(),
    );
    let mut group = c.benchmark_group("core_jwt_operations");
    group.measurement_time(Duration::from_secs(5));

    group.bench_function("create_access_token", |b| {
        b.iter(|| {
            let user_id = Uuid::new_v4();
            let token = jwt_manager.create_access_token(user_id).unwrap();
            black_box(token)
        });
    });

    group.finish();
}

fn bench_core_password_hashing(c: &mut Criterion) {
    use argon2::{
        password_hash::{PasswordHasher, SaltString},
        Argon2,
    };
    use rand::rngs::OsRng;

    let mut group = c.benchmark_group("core_password_operations");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(20); // Expensive operation

    let argon2 = Argon2::default();
    let password = "BenchmarkPassword@123";

    group.bench_function("hash_password", |b| {
        b.iter(|| {
            let salt = SaltString::generate(&mut OsRng);
            let hash = argon2.hash_password(password.as_bytes(), &salt).unwrap().to_string();
            black_box(hash)
        });
    });

    group.finish();
}

fn bench_core_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("core_serialization");
    group.measurement_time(Duration::from_secs(5));

    let user_data = json!({
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "email": "bench@test.com",
        "name": "Benchmark User",
        "created_at": "2024-01-01T00:00:00Z"
    });

    group.bench_function("serialize_user_json", |b| {
        b.iter(|| {
            let json_str = serde_json::to_string(&user_data).unwrap();
            black_box(json_str)
        });
    });

    group.finish();
}

criterion_group!(
    api_benches,
    bench_api_health_check,
    bench_api_register,
    bench_api_login,
    bench_api_list_users
);

criterion_group!(
    core_benches,
    bench_core_jwt_creation,
    bench_core_password_hashing,
    bench_core_serialization
);

criterion_main!(api_benches, core_benches);
