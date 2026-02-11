use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::testcontainers::{runners::AsyncRunner, ContainerAsync};

pub struct MockPostgres {
    // Keep container alive
    pub _container: ContainerAsync<Postgres>,
    pub connection_string: String,
    pub id: String,
}

impl MockPostgres {
    pub async fn new() -> Self {
        println!("🐳 Starting ephemeral Postgres container...");
        let container =
            Postgres::default().start().await.expect("Failed to start Postgres container");

        let host_port = container.get_host_port_ipv4(5432).await.expect("Failed to get port");
        let connection_string =
            format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", host_port);

        let id = container.id().to_string();
        println!("✅ Postgres running at {} (ID: {})", connection_string, id);

        Self { _container: container, connection_string, id }
    }
}

impl Drop for MockPostgres {
    fn drop(&mut self) {
        println!("🧹 Cleaning up Postgres container: {}", self.id);
        let _ = std::process::Command::new("docker").args(["rm", "-f", &self.id]).output();
    }
}
