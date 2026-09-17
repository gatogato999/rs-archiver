use base64::{Engine as _, engine::general_purpose::STANDARD};
use mail_parser::{MessageParser, MimeHeaders};

#[derive(Debug)]
pub struct Email {
    pub email: String,
    pub title: String,
    pub filename: String,
    pub contents: String,
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

    let attachment = message
        .attachments()
        .next()
        .ok_or_else(|| "attachment not found".to_string())?;

    let filename = attachment
        .attachment_name()
        .ok_or_else(|| "attachment filename not found".to_string())?
        .to_string();

    let contents = STANDARD.encode(attachment.contents());

    Ok(Email {
        email,
        title,
        filename,
        contents,
    })
}

#[test]
fn test_email_parser() {
    let input: &[u8] = br#"From: "Finance Dept" <finance@example.com>
To: cda@codesoft.sd
Subject: Q3 invoice
MIME-Version: 1.0
Content-Type: multipart/mixed; boundary="BOUNDARY1"

--BOUNDARY1
Content-Type: text/plain; charset="UTF-8"
Content-Transfer-Encoding: 7bit

Please find attached the Q3 invoice for review.

--BOUNDARY1
Content-Type: application/octet-stream; name="invoice.txt"
Content-Transfer-Encoding: base64
Content-Disposition: attachment; filename="invoice.txt"

VGhpcyBpcyBhIHRlc3QgYXR0YWNobWVudCBmaWxlIGZvciB0aGUgQ0RBIG1vY2sgcGlwZWxpbmUu
CkxpbmUgdHdvLgo=
--BOUNDARY1--
    "#;

    let email = parse_email(&input);
    match email {
        Ok(e) => {
            assert_eq!(e.email, "finance@example.com".to_owned());
            assert_eq!(e.title, "Q3 invoice".to_owned());
            assert_eq!(e.filename, "invoice.txt".to_owned());
            assert_eq!(
                e.contents,
                "VGhpcyBpcyBhIHRlc3QgYXR0YWNobWVudCBmaWxlIGZvciB0aGUgQ0RBIG1vY2sgcGlwZWxpbmUuCkxpbmUgdHdvLgo="
                    .to_owned()
            );
        }
        Err(e) => assert_eq!(e, "No error suppose to happen"),
    }
}
