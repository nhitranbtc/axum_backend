use crate::{
    application::actors::user_import_actor::{UserCreationActor, UserCreationMsg},
    domain::repositories::AuthRepository,
    shared::utils::password::PasswordManager,
};
use ractor::Actor;
use serde::Deserialize;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct CsvUserRecord {
    pub email: String,
    pub name: String,
    pub password: String,
}

#[derive(Debug, Error)]
pub enum ImportUsersError {
    #[error("CSV parsing error: {0}")]
    CsvError(String),
    #[error("Repository error: {0}")]
    RepositoryError(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Actor error: {0}")]
    ActorError(String),
}

pub struct ImportUsersUseCase<R: AuthRepository + 'static> {
    auth_repo: Arc<R>,
}

impl<R: AuthRepository + 'static> ImportUsersUseCase<R> {
    pub fn new(auth_repo: Arc<R>) -> Self {
        Self { auth_repo }
    }

    pub async fn execute(&self, csv_data: &[u8]) -> Result<usize, ImportUsersError> {
        let mut rdr = csv::Reader::from_reader(csv_data);
        let mut count = 0;
        let mut handles = Vec::new();

        for result in rdr.deserialize::<CsvUserRecord>() {
            let record = result.map_err(|e| ImportUsersError::CsvError(e.to_string()))?;

            // Hash password
            let password_hash = PasswordManager::hash(&record.password)
                .map_err(|e| ImportUsersError::Internal(e.to_string()))?;

            // Spawn a new actor (process) for every user
            let actor_impl = UserCreationActor::new(self.auth_repo.clone());
            let (actor_ref, handle) = Actor::spawn(None, actor_impl, ())
                .await
                .map_err(|e| ImportUsersError::ActorError(e.to_string()))?;

            // Send the message to the actor
            actor_ref
                .send_message(UserCreationMsg {
                    email: record.email,
                    name: record.name,
                    password_hash,
                })
                .map_err(|e| ImportUsersError::ActorError(e.to_string()))?;

            handles.push(handle);
            count += 1;
        }

        // Wait for all actors to finish processing
        for handle in handles {
            handle.await.map_err(|e| ImportUsersError::ActorError(e.to_string()))?;
        }

        Ok(count)
    }
}
