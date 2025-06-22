use lettre::{Message, SmtpTransport, Transport, message::{SinglePart, MultiPart, header}};
use lettre::transport::smtp::authentication::Credentials;
use std::env;

pub async fn send_code_email(email: &str, code: &str) -> Result<(), Box<dyn std::error::Error>> {
    let smtp_user = env::var("SMTP_EMAIL")?;
    let smtp_pass = env::var("SMTP_PASSWORD")?;
    let smtp_host = env::var("SMTP_SERVER")?;
    let smtp_port: u16 = env::var("SMTP_PORT")?.parse()?;

    let html_body = format!(r#"
    <div style="background-color:#6b7280;padding:50px 0">
        <div style="max-width:500px;margin:0 auto;background:#f3f4f6;padding:40px;border-radius:8px;text-align:center;font-family:Arial,sans-serif;">
            <h1 style="color:#000">Verify Your Login</h1>
            <p style="margin:20px 0;font-size:16px;color:#333">
                Use this OTP to login to your account
            </p>
            <h2 style="font-size:40px;letter-spacing:5px;color:green;margin:30px 0">{}</h2>
            <p style="color:#333">This code will securely login to your profile using<br>
            <a style="color:#3b82f6;text-decoration:none;">{}</a>
        </div>
    </div>
    "#, code, email);

    let email_message = Message::builder()
        .from(smtp_user.parse()?)
        .to(email.parse()?)
        .subject("Your OTP - Secure Login")
        .multipart(
            MultiPart::alternative()
                .singlepart(SinglePart::plain(format!("Your verification code is: {}", code)))
                .singlepart(
                    SinglePart::builder()
                        .header(header::ContentType::TEXT_HTML)
                        .body(html_body),
                ),
        )?;

    let creds = Credentials::new(smtp_user.clone(), smtp_pass);

    let mailer = SmtpTransport::starttls_relay(&smtp_host)?
        .port(smtp_port)
        .credentials(creds)
        .build();

    /*let result =*/ let _ = mailer.send(&email_message);
/*
    match result {
        Ok(_) => println!("✅ Email sent successfully"),
        Err(e) => {
            println!("❌ Failed to send email: {}", e);
            return Err(Box::new(e));
        }
    }
*/
    Ok(())
}
