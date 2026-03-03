use axum_backend::{
    grpc::{
        presentation::services::user_service::UserServiceImpl,
        proto::{user_service_client::UserServiceClient, user_service_server::UserServiceServer},
    },
    infrastructure::database::{create_scylla_session, DbPool},
};
use axum_backend::config::scylla::ScyllaConfig;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tonic::transport::{Channel, Server};
// use axum_backend::grpc::proto::user_service_client::UserServiceClient; // Removed redudant import
use crate::common::mock::MockScylla;

pub struct TestGrpcServer {
    pub addr: SocketAddr,
    pub db_pool: DbPool,
    pub _mock_db: Option<MockScylla>,
}

impl TestGrpcServer {
    pub async fn new() -> Self {
        // 1. Setup Mock DB (Reuse logic from common::server, but stripped down)
        // Or reuse MockPostgres directly
        let mock_db = MockScylla::new().await;
        
        let scylla_config = ScyllaConfig {
            nodes: vec![mock_db.contact_node.clone()],
            keyspace: format!("test_keyspace_{}", uuid::Uuid::new_v4().simple()),
            username: None,
            password: None,
            replication_factor: 1,
        };

        let pool = create_scylla_session(&scylla_config)
            .await
            .expect("Failed to create test ScyllaDB session");
        let pool = std::sync::Arc::new(pool);

        // 2. Bind to random port
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("Failed to bind");
        let addr = listener.local_addr().expect("Failed to get local addr");

        // 3. Start Server with increased actor pool for stress tests
        let user_service = UserServiceImpl::new(pool.clone(), 20)
            .await
            .expect("Failed to create user service");
        let router = Server::builder().add_service(UserServiceServer::new(user_service));

        tokio::spawn(async move {
            router
                .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
                .await
                .expect("gRPC server failed");
        });

        // Wait for server to start
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        Self { addr, db_pool: pool, _mock_db: Some(mock_db) }
    }

    pub async fn client(&self) -> UserServiceClient<Channel> {
        let uri = format!("http://{}", self.addr);
        UserServiceClient::connect(uri).await.expect("Failed to connect to test server")
    }
}
