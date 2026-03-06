use std::time::Duration;

use scylla::client::session_builder::SessionBuilder;
use testcontainers_modules::testcontainers::{
    core::ContainerPort, runners::AsyncRunner, ContainerAsync, GenericImage, ImageExt,
};

// ── Constants ────────────────────────────────────────────────────────────────

const SCYLLA_IMAGE: &str = "scylladb/scylla";
const SCYLLA_VERSION: &str = "6.2";
const SCYLLA_CQL_PORT: u16 = 9042;
const SCYLLA_DEFAULT_USER: &str = "cassandra";
const SCYLLA_DEFAULT_PASSWORD: &str = "cassandra";

/// Maximum time to wait for the CQL server to become ready after container start.
const SCYLLA_READY_TIMEOUT: Duration = Duration::from_secs(60);
/// Delay between consecutive CQL readiness probes.
const SCYLLA_RETRY_INTERVAL: Duration = Duration::from_secs(2);
/// Per-attempt connection timeout for each CQL probe.
const SCYLLA_CONNECT_TIMEOUT: Duration = Duration::from_secs(2);

// ── MockScylla ───────────────────────────────────────────────────────────────

/// An ephemeral ScyllaDB Docker container used in integration tests.
///
/// The container is started on construction and forcibly removed on [`Drop`].
pub struct MockScylla {
    /// Keeps the container alive for the lifetime of this struct.
    pub _container: ContainerAsync<GenericImage>,
    /// `host:port` of the CQL endpoint (e.g. `"127.0.0.1:32768"`).
    pub contact_node: String,
    /// Docker container ID, used for cleanup on drop.
    pub id: String,
}

impl MockScylla {
    /// Starts an ephemeral ScyllaDB container and blocks until the CQL server
    /// is fully ready to accept connections.
    ///
    /// # Panics
    /// Panics if the container fails to start or the CQL server does not become
    /// ready within [`SCYLLA_READY_TIMEOUT`].
    pub async fn new() -> Self {
        println!("🐳 Starting ephemeral ScyllaDB container...");

        let container = start_container().await;
        let host_port = container
            .get_host_port_ipv4(SCYLLA_CQL_PORT)
            .await
            .expect("Failed to resolve ScyllaDB host port");

        let contact_node = format!("127.0.0.1:{host_port}");
        let id = container.id().to_string();
        println!("✅ ScyllaDB container spawned at {contact_node} (ID: {id})");

        wait_until_cql_ready(&contact_node).await;

        Self { _container: container, contact_node, id }
    }
}

impl Drop for MockScylla {
    fn drop(&mut self) {
        println!("🧹 Cleaning up ScyllaDB container: {}", self.id);
        let _ = std::process::Command::new("docker").args(["rm", "-f", &self.id]).output();
    }
}

// ── Private helpers ──────────────────────────────────────────────────────────

/// Builds and starts the ScyllaDB Docker container.
async fn start_container() -> ContainerAsync<GenericImage> {
    GenericImage::new(SCYLLA_IMAGE, SCYLLA_VERSION)
        .with_exposed_port(ContainerPort::Tcp(SCYLLA_CQL_PORT))
        .with_cmd([
            "--smp",
            "1",
            "--memory",
            "512M",
            "--developer-mode",
            "1",
            "--authenticator",
            "PasswordAuthenticator",
        ])
        .start()
        .await
        .expect("Failed to start ScyllaDB container")
}

/// Polls the CQL endpoint until it accepts a connection or [`SCYLLA_READY_TIMEOUT`]
/// is exceeded.
///
/// Transient `ConnectionReset` errors (normal during ScyllaDB warm-up) are
/// suppressed — only a single notice is printed on the first failure.
///
/// # Panics
/// Panics on timeout.
async fn wait_until_cql_ready(contact_node: &str) {
    println!("⏳ Waiting for ScyllaDB CQL server to become ready...");

    let result = tokio::time::timeout(SCYLLA_READY_TIMEOUT, probe_until_ready(contact_node)).await;

    match result {
        Ok(()) => println!("✅ ScyllaDB CQL server is ready."),
        Err(_) => panic!(
            "Timeout: ScyllaDB CQL server at {contact_node} did not become ready within {}s",
            SCYLLA_READY_TIMEOUT.as_secs()
        ),
    }
}

/// Probes the CQL endpoint in a loop, sleeping [`SCYLLA_RETRY_INTERVAL`] between
/// attempts. Transient connection errors are silently retried; only the first
/// failure is announced so the log stays clean.
async fn probe_until_ready(contact_node: &str) {
    let mut first_failure = true;
    loop {
        match try_connect(contact_node).await {
            Ok(()) => return,
            Err(_) => {
                if first_failure {
                    println!("⏳ ScyllaDB is warming up, retrying...");
                    first_failure = false;
                }
                tokio::time::sleep(SCYLLA_RETRY_INTERVAL).await;
            },
        }
    }
}

/// Attempts a single CQL session build. Returns `Ok(())` on success.
async fn try_connect(contact_node: &str) -> Result<(), scylla::errors::NewSessionError> {
    SessionBuilder::new()
        .known_node(contact_node)
        .user(SCYLLA_DEFAULT_USER, SCYLLA_DEFAULT_PASSWORD)
        .connection_timeout(SCYLLA_CONNECT_TIMEOUT)
        .build()
        .await
        .map(|_| ())
}
