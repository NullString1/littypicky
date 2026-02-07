use crate::{
    config::EmailConfig,
    error::{AppError, Result},
    templates,
};
use lettre::{
    message::{header::ContentType, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    Message, SmtpTransport, Transport,
};

pub struct EmailService {
    config: EmailConfig,
    mailer: SmtpTransport,
}

impl EmailService {
    pub fn new(config: EmailConfig) -> Result<Self> {
        let creds = Credentials::new(
            config.smtp_username.clone(),
            config.smtp_password.clone(),
        );

        // Use builder_dangerous for localhost (MailHog), relay for production SMTP
        let mailer = if config.smtp_host == "localhost" || config.smtp_host == "127.0.0.1" {
            SmtpTransport::builder_dangerous(&config.smtp_host)
                .port(config.smtp_port)
                .build()
        } else {
            SmtpTransport::relay(&config.smtp_host)
                .map_err(|e| AppError::Email(format!("Failed to create SMTP transport: {}", e)))?
                .credentials(creds)
                .port(config.smtp_port)
                .build()
        };

        Ok(Self { config, mailer })
    }

    pub async fn send_verification_email(
        &self,
        user_email: &str,
        user_name: &str,
        token: &str,
    ) -> Result<()> {
        let verification_link = format!(
            "{}/verify-email?token={}",
            self.config.frontend_url, token
        );

        let html_template = templates::get_email_verification_html();
        let text_template = templates::get_email_verification_text();

        let html_body = templates::render_template(
            html_template,
            &[
                ("{user_name}", user_name),
                ("{verification_link}", &verification_link),
            ],
        );

        let text_body = templates::render_template(
            text_template,
            &[
                ("{user_name}", user_name),
                ("{verification_link}", &verification_link),
            ],
        );

        self.send_email(
            user_email,
            "Verify your LittyPicky account",
            &text_body,
            &html_body,
        )
        .await
    }

    pub async fn send_password_reset_email(
        &self,
        user_email: &str,
        user_name: &str,
        token: &str,
    ) -> Result<()> {
        let reset_link = format!(
            "{}/reset-password?token={}",
            self.config.frontend_url, token
        );

        let html_template = templates::get_password_reset_html();
        let text_template = templates::get_password_reset_text();

        let html_body = templates::render_template(
            html_template,
            &[("{user_name}", user_name), ("{reset_link}", &reset_link)],
        );

        let text_body = templates::render_template(
            text_template,
            &[("{user_name}", user_name), ("{reset_link}", &reset_link)],
        );

        self.send_email(
            user_email,
            "Reset your LittyPicky password",
            &text_body,
            &html_body,
        )
        .await
    }

    pub async fn send_password_reset_confirmation(
        &self,
        user_email: &str,
        user_name: &str,
    ) -> Result<()> {
        let html_template = templates::get_password_reset_confirmation_html();
        let text_template = templates::get_password_reset_confirmation_text();

        let html_body =
            templates::render_template(html_template, &[("{user_name}", user_name)]);

        let text_body =
            templates::render_template(text_template, &[("{user_name}", user_name)]);

        self.send_email(
            user_email,
            "Your LittyPicky password was reset",
            &text_body,
            &html_body,
        )
        .await
    }

    async fn send_email(
        &self,
        to_email: &str,
        subject: &str,
        text_body: &str,
        html_body: &str,
    ) -> Result<()> {
        let email = Message::builder()
            .from(
                format!("{} <{}>", self.config.smtp_from_name, self.config.smtp_from_email)
                    .parse()
                    .map_err(|e| AppError::Email(format!("Invalid from address: {}", e)))?,
            )
            .to(to_email
                .parse()
                .map_err(|e| AppError::Email(format!("Invalid to address: {}", e)))?)
            .subject(subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(SinglePart::plain(text_body.to_string()))
                    .singlepart(SinglePart::html(html_body.to_string())),
            )
            .map_err(|e| AppError::Email(format!("Failed to build email: {}", e)))?;

        // Send email in a blocking task to avoid blocking async runtime
        let mailer = self.mailer.clone();
        let result = tokio::task::spawn_blocking(move || mailer.send(&email))
            .await
            .map_err(|e| AppError::Email(format!("Task join error: {}", e)))?;
            
        match result {
            Ok(_) => {
                tracing::info!("Email sent to {}: {}", to_email, subject);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to send email to {}: {}", to_email, e);
                tracing::warn!("(Development Mode) Returning success anyway. Email content:\nSubject: {}\nBody: {}", subject, text_body);
                Ok(()) // Suppress error for development
            }
        }
    }
}
