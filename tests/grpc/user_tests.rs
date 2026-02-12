use crate::common::grpc_server::TestGrpcServer;
use axum_backend::grpc::proto::{CreateUserRequest, GetUserRequest};
use tonic::Code;

#[tokio::test]
async fn test_create_user_success() {
    let server = TestGrpcServer::new().await;
    let mut client = server.client().await;

    let request = tonic::Request::new(CreateUserRequest {
        email: "test_create@example.com".to_string(),
        name: "Test User".to_string(),
        password: "password123".to_string(),
        role: None,
    });

    let response = client.create_user(request).await.expect("Create user failed");
    let user = response.into_inner();

    assert_eq!(user.email, "test_create@example.com");
    assert_eq!(user.name, "Test User");
    assert!(!user.id.is_empty());
}

#[tokio::test]
async fn test_create_user_duplicate_email() {
    let server = TestGrpcServer::new().await;
    let mut client = server.client().await;

    // Create first user
    client.create_user(tonic::Request::new(CreateUserRequest {
        email: "duplicate@example.com".to_string(),
        name: "User 1".to_string(),
        password: "password123".to_string(),
        role: None,
    })).await.expect("First create failed");

    // Create second user with same email
    let err = client.create_user(tonic::Request::new(CreateUserRequest {
        email: "duplicate@example.com".to_string(),
        name: "User 2".to_string(),
        password: "password123".to_string(),
        role: None,
    })).await.unwrap_err();

    assert_eq!(err.code(), Code::AlreadyExists);
}

#[tokio::test]
async fn test_get_user_success() {
    let server = TestGrpcServer::new().await;
    let mut client = server.client().await;

    // Create user
    let user = client.create_user(tonic::Request::new(CreateUserRequest {
        email: "get_user@example.com".to_string(),
        name: "Get User".to_string(),
        password: "password123".to_string(),
        role: None,
    })).await.unwrap().into_inner();

    // Get user
    let fetched = client.get_user(tonic::Request::new(GetUserRequest {
        user_id: user.id.clone(),
    })).await.unwrap().into_inner();

    assert_eq!(fetched.id, user.id);
    assert_eq!(fetched.email, user.email);
}

#[tokio::test]
async fn test_get_user_not_found() {
    let server = TestGrpcServer::new().await;
    let mut client = server.client().await;

    let err = client.get_user(tonic::Request::new(GetUserRequest {
        user_id: "00000000-0000-0000-0000-000000000000".to_string(),
    })).await.unwrap_err();

    assert_eq!(err.code(), Code::NotFound);
}

#[tokio::test]
async fn test_invalid_uuid() {
    let server = TestGrpcServer::new().await;
    let mut client = server.client().await;

    let err = client.get_user(tonic::Request::new(GetUserRequest {
        user_id: "invalid-uuid".to_string(),
    })).await.unwrap_err();

    assert_eq!(err.code(), Code::InvalidArgument);
}
