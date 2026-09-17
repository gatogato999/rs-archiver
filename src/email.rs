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
            if contents.len() > 1024 {
                attachments.push(Attachement {
                    filename: file_name.to_owned(),
                    contents,
                });
            }
        }
    }

    Ok(Email {
        email,
        title,
        attachments,
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
            assert_eq!(e.attachments[0].filename, "invoice.txt".to_owned());
            assert_eq!(
                e.attachments[0].contents,
                "VGhpcyBpcyBhIHRlc3QgYXR0YWNobWVudCBmaWxlIGZvciB0aGUgQ0RBIG1vY2sgcGlwZWxpbmUuCkxpbmUgdHdvLgo="
                    .to_owned()
            );
        }
        Err(e) => assert_eq!(e, "No error suppose to happen"),
    }
}
