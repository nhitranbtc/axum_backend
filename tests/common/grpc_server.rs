use axum_backend::{
    grpc::{
        presentation::services::user_service::UserServiceImpl,
        proto::{user_service_client::UserServiceClient, user_service_server::UserServiceServer},
    },
    infrastructure::database::{
        connection::{create_pool, run_migrations},
        schema::users, // Keep if used later, but for now... no usage in CreateUser?
    },
    config::DatabaseConfig,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tonic::transport::{Channel, Server};
// use axum_backend::grpc::proto::user_service_client::UserServiceClient; // Removed redudant import
use crate::common::mock::MockPostgres;

pub struct TestGrpcServer {
    pub addr: SocketAddr,
    pub db_pool: axum_backend::infrastructure::database::connection::DbPool,
    pub _mock_db: Option<MockPostgres>,
}

impl TestGrpcServer {
    pub async fn new() -> Self {
        // 1. Setup Mock DB (Reuse logic from common::server, but stripped down)
        // Or reuse MockPostgres directly
        let mock_db = MockPostgres::new().await;
        let db_url = mock_db.connection_string.clone();
        
        let db_config = DatabaseConfig {
            max_connections: 20,  // Increased from 5 for stress tests
            min_connections: 5,   // Increased from 1
            connect_timeout: std::time::Duration::from_secs(30),
            idle_timeout: std::time::Duration::from_secs(600),
            max_lifetime: std::time::Duration::from_secs(1800),
        };
        
        let pool = create_pool(&db_config, &db_url)
            .await
            .expect("Failed to create test database pool");
            
        // Enable extensions
        {
            use diesel::sql_query;
            use diesel_async::RunQueryDsl;
            let mut conn = pool.get().await.expect("Failed to get connection for setup");
            let _ = sql_query("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";").execute(&mut conn).await;
            let _ = sql_query("CREATE EXTENSION IF NOT EXISTS \"pgcrypto\";").execute(&mut conn).await;
        }

        // Run migrations
        run_migrations(&db_url).await.expect("Failed to run migrations");

        // 2. Bind to random port
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("Failed to bind");
        let addr = listener.local_addr().expect("Failed to get local addr");

        // 3. Start Server with increased actor pool for stress tests
        let user_service = UserServiceImpl::new(pool.clone(), 20).await.expect("Failed to create user service");
        let router = Server::builder()
            .add_service(UserServiceServer::new(user_service));

        tokio::spawn(async move {
            router.serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
                .await
                .expect("gRPC server failed");
        });
        
        // Wait for server to start
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        Self {
            addr,
            db_pool: pool,
            _mock_db: Some(mock_db),
        }
    }
    
    pub async fn client(&self) -> UserServiceClient<Channel> {
        let uri = format!("http://{}", self.addr);
        UserServiceClient::connect(uri).await.expect("Failed to connect to test server")
    }
}
