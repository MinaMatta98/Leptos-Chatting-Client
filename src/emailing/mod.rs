#[cfg(feature = "ssr")]
pub mod email_client {
    use askama::Template;
    use base64::engine::{general_purpose, Engine as _};
    use lettre::message::header::ContentType;
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{Message, SmtpTransport, Transport};
    use serde::Serialize;
    use std::error::Error;

    #[derive(Template)]
    #[template(path = "template.html")]
    #[derive(Serialize)]
    struct Context<'a> {
        first_name: &'a str,
        verification_code: &'a str,
        base_64: String,
    }

    pub fn send_email(
        recipient: String,
        first_name: String,
        verification_code: String,
    ) -> Result<(), Box<dyn Error>> {
        // THIS IS A TEMPORARY DISPOSABLE PASSWORD
        let password = "aecmjqtyjexiqlpd";

        let encoded: String = general_purpose::STANDARD_NO_PAD
            .encode(std::fs::read("./assets/MagicSchoolTwo.ttf").unwrap());
        let template = Context {
            first_name: &first_name,
            verification_code: &verification_code,
            base_64: encoded,
        };
        let email = Message::builder()
            .from("ZING <jenkinssteadfast@gmail.com>".parse()?)
            .to(recipient.parse()?)
            .subject("Email Verification")
            .header(ContentType::TEXT_HTML)
            .body(template.render().unwrap())?;

        // THIS IS A TEMPORARY DISPOSABLE ACCOUNT FOR DEMO PURPOSES
        let creds = Credentials::new("jenkinssteadfast@gmail.com".to_owned(), password.to_owned());

        // Open a remote connection to gmail
        let mailer = SmtpTransport::relay("smtp.gmail.com")
            .unwrap()
            .credentials(creds)
            .build();

        // Send the email
        match mailer.send(&email) {
            Ok(_) => Ok(println!("Email sent successfully!")),
            Err(e) => panic!("Could not send email: {e:?}"),
        }
    }
}
