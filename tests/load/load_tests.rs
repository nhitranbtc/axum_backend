use crate::common::*;
use rand::Rng;
use serde_json::json;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

// --- Constants ---
const SPIKE_CONCURRENCY: usize = 1000;
const STRESS_CONCURRENCY: usize = 50;
const REQUESTS_PER_USER: usize = 5;

// --- Metrics & Statistics ---

#[derive(Debug, Clone)]
struct StressMetrics {
    total_requests: Arc<AtomicUsize>,
    successful_requests: Arc<AtomicUsize>,
    failed_requests: Arc<AtomicUsize>,
    total_duration_ms: Arc<AtomicUsize>,
}

impl StressMetrics {
    fn new() -> Self {
        Self {
            total_requests: Arc::new(AtomicUsize::new(0)),
            successful_requests: Arc::new(AtomicUsize::new(0)),
            failed_requests: Arc::new(AtomicUsize::new(0)),
            total_duration_ms: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn record_success(&self, duration_ms: u64) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.successful_requests.fetch_add(1, Ordering::Relaxed);
        self.total_duration_ms.fetch_add(duration_ms as usize, Ordering::Relaxed);
    }

    fn record_failure(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.failed_requests.fetch_add(1, Ordering::Relaxed);
    }

    fn print_summary(&self, test_name: &str) {
        let total = self.total_requests.load(Ordering::Relaxed);
        let success = self.successful_requests.load(Ordering::Relaxed);
        let failed = self.failed_requests.load(Ordering::Relaxed);
        let total_duration = self.total_duration_ms.load(Ordering::Relaxed);

        println!("\n📊 {} - Test Summary:", test_name);
        println!("=====================================");
        println!("Total Requests:      {}", total);
        if total > 0 {
            println!(
                "Successful:          {} ({:.2}%)",
                success,
                (success as f64 / total as f64) * 100.0
            );
            println!(
                "Failed:              {} ({:.2}%)",
                failed,
                (failed as f64 / total as f64) * 100.0
            );
        }
        if success > 0 {
            println!("Avg Response Time:   {} ms", total_duration / success);
        }
        println!("=====================================\n");
    }

    fn assert_success_rate(&self, min_success_rate: f64) {
        let total = self.total_requests.load(Ordering::Relaxed);
        let success = self.successful_requests.load(Ordering::Relaxed);

        if total > 0 {
            let success_rate = (success as f64 / total as f64) * 100.0;
            assert!(
                success_rate >= min_success_rate,
                "Success rate {:.2}% is below minimum {:.2}%",
                success_rate,
                min_success_rate
            );
        }
    }
}

// Stats for detailed analysis (Spike Test)
struct LoadTestStats {
    total_success: usize,
    total_failures: usize,
    latencies: Vec<u128>,
    duration: Duration,
}

impl LoadTestStats {
    fn new(duration: Duration) -> Self {
        Self { total_success: 0, total_failures: 0, latencies: Vec::new(), duration }
    }

    fn aggregate(&mut self, metrics: VUMetrics) {
        self.total_success += metrics.success_count;
        self.total_failures += metrics.failure_count;
        self.latencies.extend(metrics.latencies);
    }

    fn total_requests(&self) -> usize {
        self.total_success + self.total_failures
    }

    fn success_rate(&self) -> f64 {
        if self.total_requests() > 0 {
            (self.total_success as f64 / self.total_requests() as f64) * 100.0
        } else {
            0.0
        }
    }

    fn calculate_percentiles(&mut self) -> (f64, u128, u128) {
        self.latencies.sort_unstable();
        let len = self.latencies.len();
        if len == 0 {
            return (0.0, 0, 0);
        }
        let avg = self.latencies.iter().sum::<u128>() as f64 / len as f64;
        let p95 = self.latencies[((len as f64) * 0.95) as usize];
        let p99 = self.latencies[((len as f64) * 0.99) as usize];
        (avg, p95, p99)
    }

    fn calculate_health_score(&self, avg: f64, p95: u128) -> (f64, &'static str, f64, f64) {
        let score_reliability = (self.success_rate() / 100.0) * 40.0;
        let mut score_performance = 60.0;
        if avg > 50.0 {
            score_performance -= (avg - 50.0) / 10.0;
        }
        if p95 > 500 {
            score_performance -= 20.0;
        }
        if score_performance < 0.0 {
            score_performance = 0.0;
        }

        let total_score = score_reliability + score_performance;
        let grade = if total_score >= 90.0 {
            "A+"
        } else if total_score >= 80.0 {
            "A"
        } else if total_score >= 70.0 {
            "B"
        } else if total_score >= 50.0 {
            "C"
        } else {
            "F"
        };
        (total_score, grade, score_reliability, score_performance)
    }
}

struct VUMetrics {
    success_count: usize,
    failure_count: usize,
    latencies: Vec<u128>,
}

// --- Helpers ---

fn generate_report(stats: &mut LoadTestStats, avg: f64, p95: u128) {
    let (total_score, grade, reliability, performance) = stats.calculate_health_score(avg, p95);
    let report_content = format!(
        "# Load Test Report\nGenerated: {}\n\n| Total | Success | p95 | Score |\n|---|---|---|---|\n| {} | {:.2}% | {}ms | {:.0} ({}) |\n\nReliability: {:.1}/40 | Performance: {:.1}/60",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        stats.total_requests(), stats.success_rate(), p95, total_score, grade, reliability, performance
    );

    let report_path = "tmp/LOAD_TEST_REPORT.md";
    let _ = std::fs::create_dir_all("tmp");
    if let Ok(mut file) = std::fs::File::create(report_path) {
        let _ = file.write_all(report_content.as_bytes());
        println!("📝 Report saved to: {}", report_path);
    }
}

async fn run_virtual_user(
    id: usize,
    client: reqwest::Client,
    base_url: String,
    start_signal: tokio::sync::watch::Receiver<bool>,
    duration: Duration,
) -> VUMetrics {
    let _ = start_signal.clone().changed().await;
    let start_time = Instant::now();
    let url = format!("{}/api/auth/register", base_url);
    let mut metrics =
        VUMetrics { success_count: 0, failure_count: 0, latencies: Vec::with_capacity(100) };

    while start_time.elapsed() < duration {
        let payload = {
            let mut rng = rand::thread_rng();
            json!({
                "email": format!("load_{}_{}_{}@example.com", id, rng.gen::<u32>(), rng.gen::<u32>()),
                "name": format!("Load User {}", id),
                "password": "Password123!"
            })
        };

        let req_start = Instant::now();
        match client.post(&url).json(&payload).send().await {
            Ok(resp) if resp.status() == reqwest::StatusCode::CREATED => {
                metrics.success_count += 1;
                metrics.latencies.push(req_start.elapsed().as_millis());
            },
            _ => metrics.failure_count += 1,
        }
        sleep(Duration::from_millis(100)).await;
    }
    metrics
}

// --- Tests ---

#[tokio::test]
async fn spike_load_test() {
    println!("🚀 Starting Spike Load Test ({} users)", SPIKE_CONCURRENCY);
    let server = TestServer::new().await;
    let client = reqwest::Client::builder()
        .pool_max_idle_per_host(SPIKE_CONCURRENCY)
        .build()
        .unwrap();
    let (tx, rx) = tokio::sync::watch::channel(false);

    let duration = Duration::from_secs(30);
    let mut handles = Vec::new();

    for i in 0..SPIKE_CONCURRENCY {
        handles.push(tokio::spawn(run_virtual_user(
            i,
            client.clone(),
            server.base_url.clone(),
            rx.clone(),
            duration,
        )));
    }

    println!("🔥 Ramp-up...");
    let _ = tx.send(true);

    // Simple progress
    let start = Instant::now();
    while start.elapsed() < duration {
        sleep(Duration::from_secs(5)).await;
        println!("⏱️  {:?} elapsed...", start.elapsed());
    }

    let mut stats = LoadTestStats::new(duration);
    for handle in handles {
        if let Ok(m) = handle.await {
            stats.aggregate(m);
        } else {
            stats.total_failures += 1;
        }
    }

    let (avg, p95, p99) = stats.calculate_percentiles();
    println!(
        "Res: Total: {}, Success: {:.2}%, p95: {}ms, p99: {}ms",
        stats.total_requests(),
        stats.success_rate(),
        p95,
        p99
    );

    generate_report(&mut stats, avg, p95);

    // Soft assertion for load test
    if stats.total_failures > stats.total_requests() / 2 {
        panic!("Too many failures in load test");
    }
}

#[tokio::test]
async fn stress_test_mixed_workload() {
    println!("🚀 Starting Stress Test Mixed Workload ({} users)", STRESS_CONCURRENCY);
    let server = TestServer::new().await;

    // Setup initial user
    let email = unique_email("stress_mixed");
    server.register_user(&email, "Stress User", TEST_PASSWORD).await;
    let token = server.login_user(&email, TEST_PASSWORD).await;

    let metrics = StressMetrics::new();
    let client = server.client.clone();
    let mut handles = Vec::new();

    for i in 0..STRESS_CONCURRENCY {
        let metrics = metrics.clone();
        let client = client.clone();
        let base_url = server.base_url.clone();
        let token = token.clone();
        let email = email.clone();

        handles.push(tokio::spawn(async move {
            for j in 0..REQUESTS_PER_USER {
                let op = (i + j) % 3;
                let start = Instant::now();
                let res = match op {
                    0 => client.get(format!("{}/api/users?page=1&page_size=5", base_url))
                        .header("Authorization", format!("Bearer {}", token)).send().await,
                    1 => client.post(format!("{}/api/auth/login", base_url))
                        .json(&json!({"email": email, "password": TEST_PASSWORD})).send().await,
                    _ => client.post(format!("{}/api/auth/register", base_url))
                        .json(&json!({
                            "email": format!("stress_{}_{}_{}@test.com", i, j, rand::thread_rng().gen::<u32>()),
                            "name": "Stress", "password": TEST_PASSWORD
                        })).send().await
                };

                match res {
                    Ok(r)
                        if r.status().is_success()
                            || r.status() == reqwest::StatusCode::CREATED =>
                    {
                        metrics.record_success(start.elapsed().as_millis() as u64)
                    }
                    _ => metrics.record_failure(),
                }
            }
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    metrics.print_summary("Mixed Stress Test");
    metrics.assert_success_rate(90.0);
}
