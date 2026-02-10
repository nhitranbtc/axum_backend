use crate::application::services::email::{EmailService, EmailType, Recipient};
use crate::shared::errors::AppError;
use async_trait::async_trait;
use tracing::info;

#[derive(Clone, Default)]
pub struct NoOpEmailService;

impl NoOpEmailService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EmailService for NoOpEmailService {
    async fn send(&self, recipient: Recipient, email_type: EmailType) -> Result<(), AppError> {
        info!("(NoOp) Sending {:?} to {} <{}>", email_type, recipient.name, recipient.email);
        Ok(())
    }
}
