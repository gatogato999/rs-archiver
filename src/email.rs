use base64::{Engine as _, engine::general_purpose::STANDARD};
use mail_parser::{MessageParser, MimeHeaders};

#[derive(Debug)]
pub struct Attachement {
    pub filename: String,
    pub contents: String,
}
#[derive(Debug)]
pub struct Email {
    pub email: String,
    pub title: String,
    pub attachments: Vec<Attachement>,
}

pub fn parse_email(data: &[u8]) -> Result<Email, String> {
    let message = MessageParser::default()
        .parse(data)
        .ok_or_else(|| "failed to parse email".to_string())?;

    let email = message
        .from()
        .and_then(|addresses| addresses.first())
        .and_then(|address| address.address.as_ref())
        .map(|address| address.to_string())
        .ok_or_else(|| "sender email not found".to_string())?;

    let title = message
        .subject()
        .map(|subject| subject.to_string())
        .ok_or_else(|| "email subject not found".to_string())?;
    let mut attachments = vec![];
    for a in message.attachments() {
        if let Some(file_name) = a.attachment_name() {
            let contents = STANDARD.encode(a.contents());
            attachments.push(Attachement {
                filename: file_name.to_owned(),
                contents,
            });
        }
    }

    Ok(Email {
        email,
        title,
        attachments,
    })
}
