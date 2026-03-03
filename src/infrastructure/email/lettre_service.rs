use crate::application::services::email::{EmailService, EmailType, Recipient};
use crate::shared::errors::AppError;
use askama::Template;
use async_trait::async_trait;
use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use tracing::{error, info};

#[derive(Clone)]
pub struct LettreEmailService {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    from_email: String,
}

impl LettreEmailService {
    pub fn new() -> Result<Self, AppError> {
        let smtp_host = std::env::var("SMTP_HOST")
            .map_err(|_| AppError::Config("SMTP_HOST must be set".to_string()))?;
        let smtp_port = std::env::var("SMTP_PORT")
            .map_err(|_| AppError::Config("SMTP_PORT must be set".to_string()))?
            .parse::<u16>()
            .map_err(|e| AppError::Config(format!("Invalid SMTP_PORT: {}", e)))?;
        let smtp_user = std::env::var("SMTP_USERNAME")
            .or_else(|_| std::env::var("SMTP_USER"))
            .map_err(|_| AppError::Config("SMTP_USERNAME or SMTP_USER must be set".to_string()))?;
        let smtp_pass = std::env::var("SMTP_PASSWORD")
            .or_else(|_| std::env::var("SMTP_PASS"))
            .map_err(|_| AppError::Config("SMTP_PASSWORD or SMTP_PASS must be set".to_string()))?;
        let from_email = std::env::var("SMTP_FROM")
            .map_err(|_| AppError::Config("SMTP_FROM must be set".to_string()))?;

        let creds = Credentials::new(smtp_user.clone(), smtp_pass);

        // For production, you should use relay() and proper TLS.
        // For development/load testing, we use builder_unencrypted_localhost() or similar if no auth.
        // This is a basic implementation that assumes a standard SMTP server.
        let mailer = if smtp_host == "127.0.0.1" || smtp_host == "localhost" {
            AsyncSmtpTransport::<Tokio1Executor>::unencrypted_localhost()
        } else {
            info!("Configuring SMTP relay for {}:{} with user {}", smtp_host, smtp_port, smtp_user);
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp_host)
                .map_err(|e| {
                    AppError::Internal(anyhow::anyhow!("Failed to build SMTP transport: {}", e))
                })?
                .port(smtp_port)
                .credentials(creds)
                .build()
        };

        Ok(Self { mailer, from_email })
    }
}

#[async_trait]
impl EmailService for LettreEmailService {
    async fn send(&self, recipient: Recipient, email_type: EmailType) -> Result<(), AppError> {
        let to_address = format!("{} <{}>", recipient.name, recipient.email)
            .parse::<Mailbox>()
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid email address: {}", e)))?;

        let from_address = self
            .from_email
            .parse::<Mailbox>()
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid from address: {}", e)))?;

        let subject = email_type.subject();

        // Render template based on email type
        let body = match &email_type {
            EmailType::Welcome(name) => {
                crate::infrastructure::email::templates::WelcomeTemplate { name: name.clone() }
                    .render()
                    .map_err(|e| {
                        AppError::Internal(anyhow::anyhow!("Failed to render template: {}", e))
                    })?
            },
            EmailType::Confirmation(code) => {
                crate::infrastructure::email::templates::ConfirmationTemplate {
                    name: recipient.name.clone(),
                    code: code.clone(),
                }
                .render()
                .map_err(|e| {
                    AppError::Internal(anyhow::anyhow!("Failed to render template: {}", e))
                })?
            },
            EmailType::PasswordReset(code) => {
                crate::infrastructure::email::templates::ForgotPasswordTemplate {
                    name: recipient.name.clone(),
                    code: code.clone(),
                }
                .render()
                .map_err(|e| {
                    AppError::Internal(anyhow::anyhow!("Failed to render template: {}", e))
                })?
            },
        };

        let email = Message::builder()
            .from(from_address)
            .to(to_address)
            .subject(subject)
            .header(ContentType::TEXT_HTML) // Changed to HTML
            .body(body)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to build email: {}", e)))?;

        match self.mailer.send(email).await {
            Ok(_) => {
                info!("Email sent successfully to {}", recipient.email);
                Ok(())
            },
            Err(e) => {
                error!("Failed to send email to {}: {}", recipient.email, e);
                // We might want to return an error or just log it depending on business requirements.
                // For now, we return an error.
                Err(AppError::Internal(anyhow::anyhow!("Failed to send email: {}", e)))
            },
        }
    }
}
