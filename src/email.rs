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
            if contents.len() >= 1024 {
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
#[cfg(test)]
mod tests {
    use crate::email::parse_email;
    #[test]
    fn test_parse_basic_email_with_one_attachment() {
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

IGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIA==
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
                "IGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIA=="
                    .to_owned()
            );
            }
            Err(e) => assert_eq!(e, "No error suppose to happen"),
        }
    }
    #[test]
    fn test_parse_email_with_many_attachments() {
        let input: &[u8] = br#"From: "Finance Dept" <finance@example.com>
To: cda@codesoft.sd
Subject: Two files this time
MIME-Version: 1.0
Content-Type: multipart/mixed; boundary="BOUNDARY3"

--BOUNDARY3
Content-Type: text/plain; charset="UTF-8"
Content-Transfer-Encoding: 7bit

Two attachments in this one, on purpose.

--BOUNDARY3
Content-Type: application/octet-stream; name="report.txt"
Content-Transfer-Encoding: base64
Content-Disposition: attachment; filename="report.txt"

IGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIA==
--BOUNDARY3
Content-Type: application/octet-stream; name="appendix.txt"
Content-Transfer-Encoding: base64
Content-Disposition: attachment; filename="appendix.txt"

IGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIA==
--BOUNDARY3--
    "#;

        let email = parse_email(&input);
        match email {
            Ok(e) => {
                assert_eq!(e.email, "finance@example.com".to_owned());
                assert_eq!(e.title, "Two files this time".to_owned());
                assert_eq!(e.attachments[0].filename, "report.txt".to_owned());
                assert_eq!( e.attachments[0].contents, "IGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIA==" .to_owned());
                assert_eq!(e.attachments[1].filename, "appendix.txt".to_owned());
                assert_eq!( e.attachments[1].contents, "IGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIA==" .to_owned());
            }
            Err(e) => assert_eq!(e, "No error suppose to happen"),
        }
    }
    #[test]
    fn test_parse_email_with_no_attachments() {
        let input: &[u8] = br#"From: "Finance Dept" <finance@example.com>
To: cda@codesoft.sd
Subject: Just checking in
MIME-Version: 1.0
Content-Type: text/plain; charset="UTF-8"
Content-Transfer-Encoding: 7bit
    "#;

        let email = parse_email(&input);
        match email {
            Ok(e) => {
                assert_eq!(e.email, "finance@example.com".to_owned());
                assert_eq!(e.title, "Just checking in".to_owned());
                // email with no attch will be ignored
                assert_eq!(e.attachments.len(), 0);
            }
            Err(e) => assert_eq!(e, "No error suppose to happen"),
        }
    }
    #[test]
    fn test_parse_email_with_small_sized_attachments() {
        let input: &[u8] = br#"From: "Finance Dept" <finance@example.com>
To: cda@codesoft.sd
Subject: Just checking in
MIME-Version: 1.0
Content-Type: text/plain; charset="UTF-8"
Content-Transfer-Encoding: 7bit

--BOUNDARY3
Content-Type: application/octet-stream; name="report.txt"
Content-Transfer-Encoding: base64
Content-Disposition: attachment; filename="report.txt"

lIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtICBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSBsb3JlbSAgbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gbG9yZW0gIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIGxvcmVtIA==
    "#;

        let email = parse_email(&input);
        match email {
            Ok(e) => {
                assert_eq!(e.email, "finance@example.com".to_owned());
                assert_eq!(e.title, "Just checking in".to_owned());
                // email with small size attch will be ignored
                assert_eq!(e.attachments.len(), 0);
            }
            Err(e) => assert_eq!(e, "No error suppose to happen"),
        }
    }
}
