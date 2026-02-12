use crate::common::grpc_server::TestGrpcServer;
use axum_backend::grpc::proto::CreateUserRequest;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::task::JoinSet;

/// Performance metrics for stress test analysis
#[derive(Debug, Clone)]
struct PerformanceMetrics {
    total_requests: usize,
    successful_requests: usize,
    failed_requests: usize,
    latencies: Vec<Duration>,
    total_duration: Duration,
    test_id: String,
}

impl PerformanceMetrics {
    fn new(test_id: String) -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            latencies: Vec::new(),
            total_duration: Duration::ZERO,
            test_id,
        }
    }

    fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.successful_requests as f64 / self.total_requests as f64) * 100.0
        }
    }

    fn throughput(&self) -> f64 {
        if self.total_duration.as_secs() == 0 {
            0.0
        } else {
            self.successful_requests as f64 / self.total_duration.as_secs_f64()
        }
    }

    fn min_latency(&self) -> Option<Duration> {
        self.latencies.iter().min().copied()
    }

    fn max_latency(&self) -> Option<Duration> {
        self.latencies.iter().max().copied()
    }

    fn avg_latency(&self) -> Option<Duration> {
        if self.latencies.is_empty() {
            None
        } else {
            let sum: Duration = self.latencies.iter().sum();
            Some(sum / self.latencies.len() as u32)
        }
    }

    fn percentile(&self, p: f64) -> Option<Duration> {
        if self.latencies.is_empty() {
            return None;
        }

        let mut sorted = self.latencies.clone();
        sorted.sort();

        let index = ((p / 100.0) * sorted.len() as f64) as usize;
        let index = index.min(sorted.len() - 1);
        Some(sorted[index])
    }

    fn p50(&self) -> Option<Duration> {
        self.percentile(50.0)
    }

    fn p95(&self) -> Option<Duration> {
        self.percentile(95.0)
    }

    fn p99(&self) -> Option<Duration> {
        self.percentile(99.0)
    }

    /// Export results to CSV file
    fn export_csv(&self, results_dir: &PathBuf) -> std::io::Result<()> {
        let csv_path = results_dir.join("results.csv");
        let mut file = File::create(csv_path)?;

        writeln!(file, "status,user_id,latency_ms")?;

        for (i, latency) in self.latencies.iter().enumerate() {
            writeln!(file, "SUCCESS,{},{}", i, latency.as_millis())?;
        }

        // Add failed requests (no latency data)
        for i in 0..self.failed_requests {
            writeln!(file, "FAIL,{},0", self.successful_requests + i)?;
        }

        Ok(())
    }

    /// Generate summary report
    fn generate_summary(&self, results_dir: &PathBuf) -> std::io::Result<()> {
        let summary_path = results_dir.join("summary.txt");
        let mut file = File::create(summary_path)?;

        writeln!(file, "gRPC User Registration Stress Test Results")?;
        writeln!(file, "===========================================")?;
        writeln!(file, "Timestamp: {}", self.test_id)?;
        writeln!(file, "Host: localhost:50051")?;
        writeln!(file, "Total Users: {}", self.total_requests)?;
        writeln!(file)?;
        writeln!(file, "Results:")?;
        writeln!(file, "--------")?;
        writeln!(file, "Total Requests: {}", self.total_requests)?;
        writeln!(file, "Successful: {} ({:.1}%)", self.successful_requests, self.success_rate())?;
        writeln!(file, "Failed: {}", self.failed_requests)?;
        writeln!(file, "Duration: {}s", self.total_duration.as_secs())?;
        writeln!(file, "Throughput: {:.2} req/s", self.throughput())?;
        writeln!(file)?;
        writeln!(file, "Latency Statistics (ms):")?;
        writeln!(file, "-------------------------")?;
        writeln!(
            file,
            "Min: {}",
            self.min_latency()
                .map(|d| format!("{}ms", d.as_millis()))
                .unwrap_or_else(|| "N/A".to_string())
        )?;
        writeln!(
            file,
            "Average: {}",
            self.avg_latency()
                .map(|d| format!("{}ms", d.as_millis()))
                .unwrap_or_else(|| "N/A".to_string())
        )?;
        writeln!(
            file,
            "P50: {}",
            self.p50()
                .map(|d| format!("{}ms", d.as_millis()))
                .unwrap_or_else(|| "N/A".to_string())
        )?;
        writeln!(
            file,
            "P95: {}",
            self.p95()
                .map(|d| format!("{}ms", d.as_millis()))
                .unwrap_or_else(|| "N/A".to_string())
        )?;
        writeln!(
            file,
            "P99: {}",
            self.p99()
                .map(|d| format!("{}ms", d.as_millis()))
                .unwrap_or_else(|| "N/A".to_string())
        )?;
        writeln!(
            file,
            "Max: {}",
            self.max_latency()
                .map(|d| format!("{}ms", d.as_millis()))
                .unwrap_or_else(|| "N/A".to_string())
        )?;

        Ok(())
    }

    /// Print results to console
    fn print_results(&self) {
        println!("\n╔════════════════════════════════════════════════════════╗");
        println!("║                    TEST RESULTS                        ║");
        println!("╚════════════════════════════════════════════════════════╝\n");

        println!("📊 Summary:");
        println!("   ✅ Successful: {} ({:.1}%)", self.successful_requests, self.success_rate());
        println!("   ❌ Failed: {}", self.failed_requests);
        println!("   ⏱️  Duration: {}s", self.total_duration.as_secs());
        println!("   🚀 Throughput: {:.2} req/s", self.throughput());
        println!();

        if let (Some(min), Some(avg), Some(max)) =
            (self.min_latency(), self.avg_latency(), self.max_latency())
        {
            println!("⏱️  Latency (ms):");
            println!(
                "   Min: {}ms | Avg: {}ms | Max: {}ms",
                min.as_millis(),
                avg.as_millis(),
                max.as_millis()
            );

            if let (Some(p50), Some(p95), Some(p99)) = (self.p50(), self.p95(), self.p99()) {
                println!(
                    "   P50: {}ms | P95: {}ms | P99: {}ms",
                    p50.as_millis(),
                    p95.as_millis(),
                    p99.as_millis()
                );
            }
            println!();
        }

        println!("🎯 Performance Targets:");

        // Success rate target: >= 95%
        if self.success_rate() >= 95.0 {
            println!("   Success Rate: ✅ PASS");
        } else {
            println!("   Success Rate: ❌ FAIL");
        }

        // P95 latency target: < 5000ms
        if let Some(p95) = self.p95() {
            if p95.as_millis() < 5000 {
                println!("   P95 Latency: ✅ PASS");
            } else {
                println!("   P95 Latency: ❌ FAIL");
            }
        }

        // Throughput target: >= 50 req/s
        if self.throughput() >= 50.0 {
            println!("   Throughput: ✅ PASS");
        } else {
            println!("   Throughput: ⚠️  WARN");
        }
    }
}

/// Run stress test with configurable parameters
async fn run_stress_test(
    num_users: usize,
    concurrent: usize,
    test_name: &str,
) -> PerformanceMetrics {
    // Generate test ID from timestamp
    let test_id = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_string();

    // Create results directory
    let results_dir =
        PathBuf::from("/tmp/grpc-stress-test").join(format!("{}_{}", test_name, test_id));
    std::fs::create_dir_all(&results_dir).expect("Failed to create results directory");

    println!("\n╔════════════════════════════════════════════════════════╗");
    println!("║     gRPC User Registration Stress Test (Rust)         ║");
    println!("╚════════════════════════════════════════════════════════╝\n");
    println!("Configuration:");
    println!("  • Total Users: {}", num_users);
    println!("  • Concurrent Requests: {}", concurrent);
    println!("  • Results Directory: {}", results_dir.display());
    println!("  • Test ID: {}\n", test_id);

    // Initialize test server
    let server = TestGrpcServer::new().await;

    let mut metrics = PerformanceMetrics::new(test_id.clone());
    metrics.total_requests = num_users;

    let start_time = Instant::now();

    // Process in batches to control concurrency
    for batch_start in (0..num_users).step_by(concurrent) {
        let batch_end = (batch_start + concurrent).min(num_users);
        let mut join_set = JoinSet::new();

        for i in batch_start..batch_end {
            let mut client = server.client().await;
            let test_id_clone = test_id.clone();

            join_set.spawn(async move {
                let request_start = Instant::now();

                let request = tonic::Request::new(CreateUserRequest {
                    email: format!("stress_test_{}_{:06}@example.com", test_id_clone, i),
                    name: format!("Stress Test User {}", i),
                    password: "SecurePassword123!".to_string(),
                    role: None,
                });

                let result = client.create_user(request).await;
                let latency = request_start.elapsed();

                (i, result, latency)
            });
        }

        // Collect batch results
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok((user_id, create_result, latency)) => match create_result {
                    Ok(_) => {
                        metrics.successful_requests += 1;
                        metrics.latencies.push(latency);

                        if metrics.successful_requests % 100 == 0 {
                            println!(
                                "✅ Progress: {}/{} users created",
                                metrics.successful_requests, num_users
                            );
                        }
                    },
                    Err(e) => {
                        metrics.failed_requests += 1;
                        eprintln!("❌ Failed to create user {}: {:?}", user_id, e);
                    },
                },
                Err(e) => {
                    metrics.failed_requests += 1;
                    eprintln!("❌ Task join error: {:?}", e);
                },
            }
        }
    }

    metrics.total_duration = start_time.elapsed();

    // Export results
    metrics.export_csv(&results_dir).expect("Failed to export CSV");
    metrics.generate_summary(&results_dir).expect("Failed to generate summary");

    // Print results
    metrics.print_results();

    println!("\n📁 Results saved to: {}", results_dir.display());
    println!("   • results.csv - Detailed per-request data");
    println!("   • summary.txt - Test summary report\n");

    metrics
}

/// Baseline Test: 10% capacity, 5 minutes
#[tokio::test]
#[ignore]
async fn test_baseline_100_users() {
    let metrics = run_stress_test(100, 10, "baseline_test").await;

    // Assert baseline targets
    assert!(
        metrics.success_rate() >= 95.0,
        "Success rate too low: {:.1}%",
        metrics.success_rate()
    );
}

/// Load Test: 100% capacity
#[tokio::test]
#[ignore]
async fn test_load_1000_users() {
    let metrics = run_stress_test(1000, 100, "load_test").await;

    // Assert load test targets
    assert!(
        metrics.success_rate() >= 95.0,
        "Success rate too low: {:.1}%",
        metrics.success_rate()
    );
    assert!(
        metrics.throughput() >= 50.0,
        "Throughput too low: {:.2} req/s",
        metrics.throughput()
    );
}

/// Stress Test: High concurrency
#[tokio::test]
#[ignore]
async fn test_stress_5000_users() {
    let metrics = run_stress_test(5000, 500, "stress_test").await;

    // Stress test may have lower success rate
    assert!(
        metrics.success_rate() >= 80.0,
        "Success rate too low: {:.1}%",
        metrics.success_rate()
    );
}

/// Spike Test: Sudden high load
#[tokio::test]
#[ignore]
async fn test_spike_2000_users() {
    let metrics = run_stress_test(2000, 200, "spike_test").await;

    assert!(
        metrics.success_rate() >= 90.0,
        "Success rate too low: {:.1}%",
        metrics.success_rate()
    );
}

/// Quick smoke test
#[tokio::test]
#[ignore]
async fn test_smoke_50_users() {
    let metrics = run_stress_test(50, 10, "smoke_test").await;

    assert!(
        metrics.success_rate() >= 99.0,
        "Success rate too low: {:.1}%",
        metrics.success_rate()
    );
}

/// Endurance test: Long duration
#[tokio::test]
#[ignore]
async fn test_endurance_10000_users() {
    let metrics = run_stress_test(10000, 100, "endurance_test").await;

    assert!(
        metrics.success_rate() >= 95.0,
        "Success rate too low: {:.1}%",
        metrics.success_rate()
    );
}
