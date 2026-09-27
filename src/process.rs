use crate::{
    email::parse_email,
    storage::{AttachmentKey, AttachmentStatus, Store},
    upload::{UploadReq, Uploader},
};
pub fn process_messages(
    client: &mut pop3_mail_client::Pop3Client,
    cda_endpoint: &str,
    uploader_user_name: String,
    uploader_user_pass: String,
    app_domain: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::load_or_create("state.json")?;
    let uploader = Uploader::new();
    let uidl_response = match client.uidl() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("uidl() failed: {e}");
            return Ok(());
        }
    };
    for item in uidl_response.messages {
        let message_number = item.message_id;
        let uidl = item.unique_id;
        println!("Processing message {}, UIDL {}", message_number, uidl);
        let msg = match client.retrieve_as_string(message_number) {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("skip: client failed to retrieve_as_string {message_number}: {e}");
                continue;
            }
        };

        let email = match parse_email(&msg.data) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("skip: parse failed for message {message_number}: {e}");
                continue;
            }
        };

        let attachment_names: Vec<String> = email
            .attachments
            .iter()
            .map(|a| a.filename.clone())
            .collect();
        for attachment in &email.attachments {
            let key = AttachmentKey {
                uidl: uidl.clone(),
                attachment: attachment.filename.clone(),
            };

            if !store.is_uploadable(&key) {
                println!("skipping already-resolved: {}", attachment.filename);
                continue;
            }

            let status: AttachmentStatus = uploader.call_upload(
                cda_endpoint,
                UploadReq {
                    username: uploader_user_name.clone(),
                    password: uploader_user_pass.clone(),
                    email: email.email.clone(),
                    domain: app_domain.clone(),
                    title: email.title.clone(),
                    filename: attachment.filename.clone(),
                    contents: attachment.contents.clone(),
                },
            );
            if let Err(e) = store.record_outcome(key, status) {
                println!(
                    "record_outcome failed att-name: {} msg-uidl:{uidl} : {e}",
                    attachment.filename
                )
            }
        }
        if attachment_names.is_empty() {
            println!("skipp msg with no attch : uidl={uidl}");
        } else if store.message_is_fully_resolved(uidl.as_str(), &attachment_names) {
            match client.delete(message_number) {
                Ok(_) => println!("deleted message {message_number} (uidl {uidl})"),
                Err(e) => eprintln!("failed to del message {message_number} (uidl {uidl}): {e}"),
            }
        } else {
            println!(
                "leaving message {message_number} (uidl {uidl}, {}) -- still has retryable work",
                email.email
            );
        }
    }
    Ok(())
}
