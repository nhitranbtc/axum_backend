use axum_backend::domain::entities::User;
use axum_backend::domain::repositories::user::{UserRepository, RepositoryError};
use axum_backend::domain::value_objects::{Email, UserId};
use axum_backend::infrastructure::cache::{CacheRepository, CacheError};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// Mock User Repository
pub struct MockUserRepository {
    pub users: Arc<Mutex<Vec<User>>>,
}

impl MockUserRepository {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl UserRepository for MockUserRepository {
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, RepositoryError> {
        let users = self.users.lock().unwrap();
        Ok(users.iter().find(|u| u.email == *email).cloned())
    }

    async fn exists_by_email(&self, email: &Email) -> Result<bool, RepositoryError> {
        let users = self.users.lock().unwrap();
        Ok(users.iter().any(|u| u.email == *email))
    }

    async fn save(&self, user: &User) -> Result<User, RepositoryError> {
        let mut users = self.users.lock().unwrap();
        users.push(user.clone());
        Ok(user.clone())
    }

    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepositoryError> {
        let users = self.users.lock().unwrap();
        Ok(users.iter().find(|u| u.id == id).cloned())
    }

    async fn update(&self, user: &User) -> Result<User, RepositoryError> {
        let mut users = self.users.lock().unwrap();
        if let Some(pos) = users.iter().position(|u| u.id == user.id) {
            users[pos] = user.clone();
            Ok(user.clone())
        } else {
             Err(RepositoryError::NotFound)
        }
    }

    async fn count(&self) -> Result<i64, RepositoryError> {
        Ok(0)
    }

    async fn list_paginated(&self, _limit: i64, _offset: i64) -> Result<Vec<User>, RepositoryError> {
        Ok(vec![])
    }

    async fn delete(&self, _id: UserId) -> Result<bool, RepositoryError> {
        Ok(true)
    }

    async fn delete_all(&self) -> Result<usize, RepositoryError> {
        Ok(0)
    }
}

// Mock Cache Repository (noop)
pub struct MockCacheRepository;

#[async_trait]
impl CacheRepository for MockCacheRepository {
    async fn get(&self, _key: &str) -> Result<Option<String>, CacheError> {
        Ok(None)
    }
    async fn set(&self, _key: &str, _value: &str, _ttl: Duration) -> Result<(), CacheError> {
        Ok(())
    }
    async fn delete(&self, _key: &str) -> Result<(), CacheError> {
        Ok(())
    }
    async fn set_nx(&self, _key: &str, _value: &str, _ttl: Duration) -> Result<bool, CacheError> {
        Ok(true)
    }
    async fn delete_if_equals(&self, _key: &str, _value: &str) -> Result<bool, CacheError> {
        Ok(true)
    }
}

