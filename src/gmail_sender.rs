use anyhow::Result;
use lettre::{
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

pub async fn send_email(to: &str, subject: &str, body: &str) -> Result<()> {
    let user = std::env::var("GMAIL_USER")?;
    let pass = std::env::var("GMAIL_APP_PASSWORD")?.replace(' ', "");

    let email = Message::builder()
        .from(user.parse()?)
        .to(to.parse()?)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body.to_string())?;

    let creds = Credentials::new(user, pass);

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")?
        .credentials(creds)
        .build();

    mailer.send(email).await?;
    tracing::info!("Email sent to {}", to);
    Ok(())
}
